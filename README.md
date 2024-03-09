# Aeolus

<h6>Ancient Greek: Αἴολος, Aiolos - In Greek mythology, Aeolus the son of Hippotes, was the ruler of the winds.</h6>

This is a proof-of-concept (PoC) implementation of Unimog.

## Description:

**<u>Note:</u>** Since this is a PoC, to make it simpler, Aeolus follows a slightly different approach to transfer packets between servers than Unimog.

### Setup consideration:

For the purpose of testing Aeolus I created two Linux virtual machines(VM), and manually added the same IP address to each of these VMs. And in addition to that IP, each server has a unique IP that will be used for health check.

### Transfering packets between servers:

Since all servers share the same IP, I opted to transfer packets between servers by modifying the MAC address of the packet rather than encapsulating the packet with the generic UDP encapsulation method.

## Usage:

```
Usage: aeolus [OPTIONS]

Options:
      --config <FILE>   Path to Aeolus configuration file [default: aeolus.yaml]
  -h, --help            Print help
```

### Configuration File:

#### Options:

ports: A list of u16 values. (Optional - Defaults to `[80]`)
servers: A list of dictinaries with `ip_address` and 'mac_address' keys.
logfile: Path to the log file. (Optional - Defaults to `/var/log/aeolus.log`)
iface: The name of the interface to attach the xdp app to. (Optional - Defaults to `wlp1s0`)

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
```
