# Aeolus

<h6>Ancient Greek: Αἴολος, Aiolos - In Greek mythology, Aeolus was the ruler of the winds.</h6>

This is a proof-of-concept (PoC) implementation of Unimog.

## Description:

**<u>Note:</u>** Since this is a PoC, to make it simpler, Aeolus follows a slightly different approach to transfer packets between servers compared to Unimog.

### Setup consideration:

For the purpose of testing Aeolus and to simplify the architecture, I created two Linux virtual machines, and manually added a matching IP address from within the machines. And in addition to that IP, each machine has its own unique IP which is used for health check. However, this is not a recommended setup because the address resolution protocol (ARP) will make all packets arrive on the same machine before being transfered or consumed, causing a single entry point.

### Transfering packets between servers:

Since all servers share the same IP, I opted to transfer packets between servers by modifying the destination MAC address of the packet rather than encapsulating it with the generic UDP encapsulation method.

## Usage:

```
Usage: aeolus [OPTIONS]

Options:
      --config <FILE>   Path to Aeolus configuration file [default: aeolus.yaml]
  -h, --help            Print help
```

### Configuration File:

#### Options:

- `servers`: A list of dictionaries with `ip` and `mac` keys.
- `ports`: A list of u16 values. [Optional - Defaults to `[80]`]
- `logfile`: Path to the log file. [Optional - Defaults to `/var/log/aeolus.log`]
- `iface`: The name of the interface to attach the xdp app to. [Optional - Defaults to `wlp1s0`]
- `health_interval`: A u8 value representing how often should the health checker run. (In seconds) [Optional - Defaults to `10`]

#### Sample aeolus.yaml file:
```YAML
servers:
  - mac: 52:54:00:94:df:40
    ip: 192.168.122.246

  - mac: 52:54:00:61:be:9e
    ip: 192.168.122.72

ports: 
  - 80
  - 443

logfile: ./aeolus.log
iface: enp3s0
health_interval: 30
```
<h6>The file configures Aeolus to only balance ports <i>80</i> and <i>443</i> between two servers, logs everything in <i>./aeolus.log</i>, and attaches the xdp application to interface <i>enp3s0</i>.</h6>
