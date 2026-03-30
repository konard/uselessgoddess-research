# Topic 9: Server-Worker Communication Architecture

## 1. Protocol Comparison: REST vs gRPC vs MQTT vs WebSocket

| Feature | REST (HTTP) | gRPC | MQTT | WebSocket |
|---|---|---|---|---|
| Protocol | HTTP/1.1–2 | HTTP/2 | TCP (MQTT) | TCP (WS) |
| Message format | JSON/XML | Protobuf (binary) | Binary/JSON | Any |
| Streaming | Limited (SSE, polling) | Native bidirectional | Native pub/sub | Bidirectional |
| Overhead | Medium (HTTP headers) | Low (binary frames) | Very low | Low |
| Worker→Server push | Polling or webhook | Yes (streaming) | Yes (publish) | Yes |
| Broker required | No | No | Yes (MQTT broker) | No |
| Self-hosted ease | Easy (any HTTP server) | Moderate | Easy (Mosquitto) | Moderate |
| Ideal for | Simple CRUD, human-readable | High-performance RPC | IoT, many devices | Real-time bidirectional |
| Best fit for this system | Simple, good for small fleet | Best for large fleet | Excellent for 10–50 workers | Good for real-time commands |

**Recommendation:** For 10–50 workers, **MQTT** or **gRPC** is optimal.

### MQTT (Message Queue Telemetry Transport)

MQTT is the IoT standard — exactly the pattern of a central server sending commands to many edge workers.

**Architecture:**
```
Server → publishes to MQTT broker → workers subscribe
Workers → publish status/results → server subscribes
```

Self-hosted broker: **Mosquitto** (very lightweight, ~5 MB)

```bash
# Install Mosquitto
sudo apt install mosquitto mosquitto-clients

# Worker subscribes to commands
mosquitto_sub -h broker -t "workers/worker-01/commands/#"

# Server sends command to specific worker
mosquitto_pub -h broker -t "workers/worker-01/commands/start" \
  -m '{"account": "user@example.com", "password": "pass123"}'

# Worker reports status
mosquitto_pub -h broker -t "workers/worker-01/status" \
  -m '{"state": "farming", "vm_count": 3, "uptime": 3600}'
```

Python MQTT client (paho-mqtt):
```python
import paho.mqtt.client as mqtt
import json

class WorkerAgent:
    def __init__(self, broker, worker_id):
        self.worker_id = worker_id
        self.client = mqtt.Client()
        self.client.on_message = self.on_command
        self.client.connect(broker)
        self.client.subscribe(f"workers/{worker_id}/commands/#")
        self.client.loop_start()

    def on_command(self, client, userdata, msg):
        command = json.loads(msg.payload)
        topic = msg.topic.split('/')[-1]

        if topic == 'start_vm':
            self.start_vm(command['account'], command['vm_config'])
        elif topic == 'stop_vm':
            self.stop_vm(command['vm_id'])

    def report_status(self, status):
        self.client.publish(
            f"workers/{self.worker_id}/status",
            json.dumps(status)
        )

    def start_vm(self, account, vm_config):
        # Create and start VM, launch CS2
        pass
```

### gRPC (for High-Performance Large Fleets)

```protobuf
// worker.proto
syntax = "proto3";

service WorkerService {
  rpc StartAccount (StartRequest) returns (StartResponse);
  rpc StopAccount (StopRequest) returns (StopResponse);
  rpc GetStatus (StatusRequest) returns (stream StatusUpdate);
}

message StartRequest {
  string account_id = 1;
  string steam_username = 2;
  string steam_password = 3;
  VMConfig vm_config = 4;
}

message VMConfig {
  int32 ram_mb = 1;
  int32 vcpu_count = 2;
  string smbios_manufacturer = 3;
  string disk_serial = 4;
  string mac_address = 5;
}

message StatusUpdate {
  string worker_id = 1;
  int32 active_vms = 2;
  string state = 3;
  int64 timestamp = 4;
}
```

### Redis Pub/Sub (Simple, Low Dependencies)

```python
import redis
import json
import threading

class WorkerAgent:
    def __init__(self, redis_host, worker_id):
        self.worker_id = worker_id
        self.r = redis.Redis(host=redis_host)
        self.pubsub = self.r.pubsub()
        self.pubsub.subscribe(f"worker:{worker_id}:commands")

    def listen(self):
        for message in self.pubsub.listen():
            if message['type'] == 'message':
                cmd = json.loads(message['data'])
                self.handle_command(cmd)

    def report_status(self, status):
        self.r.set(f"worker:{self.worker_id}:status", json.dumps(status))
        self.r.publish("server:worker_status", json.dumps({
            "worker_id": self.worker_id,
            **status
        }))

    def handle_command(self, cmd):
        if cmd['action'] == 'start_vm':
            self.start_vm(cmd['account'])
```

## 2. Account Queue Management

### Redis-based job queue:

```python
import redis
import json

class AccountQueue:
    def __init__(self, redis_host):
        self.r = redis.Redis(host=redis_host)

    # Server: enqueue account for farming
    def enqueue(self, account):
        self.r.rpush('accounts:queue', json.dumps(account))

    # Worker: claim next account
    def claim_account(self, worker_id, timeout=30):
        # Atomic pop from queue into worker-specific processing list
        result = self.r.blmove(
            'accounts:queue',
            f'accounts:processing:{worker_id}',
            timeout=timeout
        )
        return json.loads(result) if result else None

    # Worker: mark account as done
    def complete_account(self, worker_id, account_id):
        self.r.lrem(f'accounts:processing:{worker_id}', 1,
                   json.dumps({'id': account_id}))
        self.r.rpush('accounts:done', account_id)
```

### Handling Worker Reconnection

```python
# On worker startup, re-claim any accounts in processing list
# (handles crash recovery)
def recover_on_start(self, worker_id):
    processing = self.r.lrange(f'accounts:processing:{worker_id}', 0, -1)
    for account_data in processing:
        account = json.loads(account_data)
        self.start_vm(account)  # Resume farming
```

## 3. Worker Heartbeat and Health Reporting

```python
import threading
import time

class HeartbeatWorker:
    def __init__(self, client, worker_id, interval=30):
        self.worker_id = worker_id
        self.interval = interval
        thread = threading.Thread(target=self._heartbeat_loop, daemon=True)
        thread.start()

    def _heartbeat_loop(self):
        while True:
            self.report_status({
                "worker_id": self.worker_id,
                "state": "online",
                "active_vms": self.get_active_vm_count(),
                "cpu_percent": self.get_cpu_usage(),
                "ram_available_mb": self.get_available_ram(),
                "timestamp": time.time()
            })
            time.sleep(self.interval)
```

Server side: workers that don't heartbeat in 2x interval are marked offline, their accounts returned to queue.

## 4. Authentication Between Server and Workers

### API Key (Simple, good for trusted network)
```python
# Worker registers with server
headers = {"Authorization": f"Bearer {WORKER_API_KEY}"}
```

### mTLS (Mutual TLS) — for untrusted networks
```bash
# Generate worker certificate
openssl req -newkey rsa:4096 -nodes -keyout worker.key \
  -out worker.csr -subj "/CN=worker-01"
# Sign with CA
openssl x509 -req -in worker.csr -CA ca.crt -CAkey ca.key \
  -out worker.crt -days 365
```

### JWT
```python
import jwt
token = jwt.encode({"worker_id": worker_id, "exp": time.time() + 3600},
                   SECRET_KEY, algorithm="HS256")
```

## 5. Recommended Architecture for 10–50 Workers

```
Central Server (Python/FastAPI or Go)
    ↕ MQTT (Mosquitto) or Redis Pub/Sub
Worker Agents (Python daemon on each host)
    ↕ libvirt API
KVM/QEMU Virtual Machines
    ↕ VNC / QMP
CS2 instances inside VMs
```

**Component choices:**
- **Message broker:** Mosquitto MQTT (2 MB RAM, battle-tested, IoT standard)
- **Job queue:** Redis (in-memory, fast, atomic operations)
- **Worker agent:** Python daemon with paho-mqtt + libvirt-python
- **Server:** FastAPI (Python) for REST admin API + MQTT for worker control
- **Monitoring:** Prometheus + libvirt-exporter + Grafana

## Sources
- https://mosquitto.org/
- https://redis.io/docs/manual/pubsub/
- https://grpc.io/docs/languages/python/
- https://pypi.org/project/paho-mqtt/
- https://fastapi.tiangolo.com/
