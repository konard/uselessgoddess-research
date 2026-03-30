# Topic 6: Input Injection and Display Capture for KVM VMs

## 1. QEMU QMP — Keyboard/Mouse Input Injection

QMP (QEMU Machine Protocol) exposes a JSON socket for controlling QEMU.

**Enable QMP in VM definition:**
```xml
<domain type='kvm' xmlns:qemu='http://libvirt.org/schemas/domain/qemu/1.0'>
  <qemu:commandline>
    <qemu:arg value='-qmp'/>
    <qemu:arg value='unix:/tmp/vm-qmp.sock,server=on,wait=off'/>
  </qemu:commandline>
</domain>
```

Or in libvirt XML:
```xml
<channel type='unix'>
  <source mode='bind' path='/tmp/vm-qmp.sock'/>
  <target type='virtio' name='org.qemu.guest_agent.0'/>
</channel>
```

### QMP Commands for Input

```python
import json
import socket

def send_qmp(sock_path, command, arguments=None):
    with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as s:
        s.connect(sock_path)
        # Read capabilities
        caps = json.loads(s.recv(4096))
        # Negotiate
        s.send(json.dumps({"execute": "qmp_capabilities"}).encode())
        s.recv(4096)
        # Send command
        msg = {"execute": command}
        if arguments:
            msg["arguments"] = arguments
        s.send(json.dumps(msg).encode())
        return json.loads(s.recv(4096))

# Move mouse to (100, 200)
send_qmp('/tmp/vm-qmp.sock', 'input-send-event', {
    "events": [
        {"type": "abs", "data": {"axis": "x", "value": 100}},
        {"type": "abs", "data": {"axis": "y", "value": 200}}
    ]
})

# Left mouse click (press + release)
send_qmp('/tmp/vm-qmp.sock', 'input-send-event', {
    "events": [{"type": "btn", "data": {"down": True, "button": "left"}}]
})
send_qmp('/tmp/vm-qmp.sock', 'input-send-event', {
    "events": [{"type": "btn", "data": {"down": False, "button": "left"}}]
})

# Key press (Enter)
send_qmp('/tmp/vm-qmp.sock', 'input-send-event', {
    "events": [
        {"type": "key", "data": {"down": True, "key": {"type": "qcode", "data": "ret"}}},
        {"type": "key", "data": {"down": False, "key": {"type": "qcode", "data": "ret"}}}
    ]
})
```

### Screendump via QMP

```python
# Take screenshot and save to file
send_qmp('/tmp/vm-qmp.sock', 'screendump', {"filename": "/tmp/frame.ppm"})

# Convert PPM to usable format with Pillow
from PIL import Image
img = Image.open('/tmp/frame.ppm')
img.save('/tmp/frame.png')
```

Python library for async QMP: `pip install qemu.qmp`

## 2. libvirt Python API

```python
import libvirt

conn = libvirt.open('qemu:///system')
dom = conn.lookupByName('worker-01')

# Take screenshot
import io
stream = conn.newStream()
dom.screenshot(stream, 0)
buf = io.BytesIO()
stream.recv(b'', 65536)  # Read PPM data
```

Full Python example:
```python
import libvirt
from PIL import Image
import io

def get_screenshot(domain_name):
    conn = libvirt.open('qemu:///system')
    dom = conn.lookupByName(domain_name)

    # Get screenshot as PPM
    stream = conn.newStream(libvirt.VIR_STREAM_NONBLOCK)
    mime = dom.screenshot(stream, 0)

    buf = b''
    while True:
        try:
            data = stream.recv(65536)
            if not data:
                break
            buf += data
        except:
            break
    stream.finish()
    conn.close()

    return Image.open(io.BytesIO(buf))
```

## 3. VNC Framebuffer Capture

QEMU built-in VNC server (add to VM definition):
```xml
<graphics type='vnc' port='-1' autoport='yes' listen='127.0.0.1'>
  <listen type='address' address='127.0.0.1'/>
</graphics>
```

Python VNC client with `vncdotool`:
```bash
pip install vncdotool
```

```python
from vncdotool import api

# Connect to VM VNC
client = api.connect('127.0.0.1', password=None, port=5900)

# Take screenshot
client.captureScreen('/tmp/screenshot.png')

# Move mouse and click
client.mouseMove(x=100, y=200)
client.mousePress(1)  # left button

# Type text
client.type('text to type')

# Press key
client.keyPress('Return')
client.keyPress('ctrl-a')  # Ctrl+A

client.disconnect()
```

`vncdotool` supports both one-shot and continuous framebuffer watching.

## 4. virtio-input (Alternative Input Device)

```xml
<input type='mouse' bus='virtio'/>
<input type='keyboard' bus='virtio'/>
<input type='tablet' bus='virtio'/>
```

`virtio-tablet` uses **absolute coordinates** (0 to 0x7FFF range), which is often better for programmatic control than relative mouse movements.

The `tablet` device sends absolute X/Y coordinates to the guest — no cursor drift issues.

## 5. Practical Automation Architecture

### Recommended: VNC + OpenCV

```python
import cv2
import numpy as np
from vncdotool import api
from PIL import Image

class VMController:
    def __init__(self, host, port=5900):
        self.client = api.connect(host, port=port)

    def get_frame(self):
        """Get current VM frame as numpy array for OpenCV"""
        self.client.captureScreen('/tmp/frame.png')
        img = cv2.imread('/tmp/frame.png')
        return img

    def click(self, x, y):
        self.client.mouseMove(x, y)
        self.client.mousePress(1)
        self.client.mouseRelease(1)

    def key_press(self, key):
        self.client.keyPress(key)

    def find_template(self, template_path):
        """Find template image in VM frame, return (x, y) or None"""
        frame = self.get_frame()
        template = cv2.imread(template_path)
        result = cv2.matchTemplate(frame, template, cv2.TM_CCOEFF_NORMED)
        _, max_val, _, max_loc = cv2.minMaxLoc(result)
        if max_val > 0.8:  # threshold
            h, w = template.shape[:2]
            return (max_loc[0] + w//2, max_loc[1] + h//2)
        return None

    def disconnect(self):
        self.client.disconnect()
```

### AI Inference Pipeline

```python
# Frame → inference → command
frame = controller.get_frame()
# Run YOLO or similar model on frame
detections = model.predict(frame)
# Parse detections → decide action
# Execute action via VNC
controller.click(target_x, target_y)
```

## 6. egl-headless Display (No X11 Required)

For headless operation without X server:

```bash
qemu-system-x86_64 \
  -display egl-headless \
  -vnc 127.0.0.1:0 \
  ...
```

This allows screendump via QMP or VNC without any X11/Wayland on host.

## Summary: Best Approach

| Use Case | Recommended Method |
|---|---|
| One-shot screenshots | QEMU QMP `screendump` |
| Continuous framebuffer stream | VNC client library (vncdotool) |
| Mouse/keyboard injection | VNC `mouseMove`/`keyPress` or QMP `input-send-event` |
| AI vision + control | VNC capture → OpenCV/YOLO → VNC input |
| Batch automation scripting | vncdotool high-level API |

## Sources
- https://qemu-project.gitlab.io/qemu/interop/qemu-qmp-ref.html
- https://www.libvirt.org/python
- https://github.com/sibson/vncdotool
- https://wiki.archlinux.org/title/QEMU
