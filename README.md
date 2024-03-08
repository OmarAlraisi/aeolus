# Aeolus

<h6>Ancient Greek: Αἴολος, Aiolos - In Greek mythology, Aeolus the son of Hippotes, was the ruler of the winds.</h6>

This is a proof-of-concept (PoC) implementation of Unimog.

## Description:

**<u>Note:</u>** Since this is a PoC, to make it simpler, Aeolus follows a slightly different approach to transfer packets between servers than Unimog.

### Setup:

For the purpose of testing Aeolus I created two Linux virtual machines(VM), and manually added the same IP address to each of these VMs.

### Transfering packets between servers:

Since all servers share the same IP, I opted to transfer packets between servers by modifying the MAC address of the packet rather than encapsulating the packet with the generic UDP encapsulation method. Therefore, unlike Unimog, Aeolus takes MAC addresses to configure the load balancer rather than direct IP addresses.

## Usage:

```
Usage: aeolus [OPTIONS]

Options:
  -s, --servers <MAC>             Comma separated servers' MAC addresses
  -p, --ports <PORT>              Comma separated ports [default: 80]
  -i, --iface <NI>                Netowrk interface to attach eBPF app to [default: wlp1s0]
      --logfile <FILE>            Path to log file [default: /var/log/aeolus.log]
      --config <FILE>             Path to Aeolus configuration file
  -h, --help                      Print help
```

**<u>Note:</u>** Conflicting configurations will cause an error. (i.e. cannot specify `-s`, `-p`, or `--logfile` while also specifying `--config`).

**<u>Sample Configuration File:</u>**

```YAML
ports: 
  - 80
  - 443
servers:
  - 00:00:00:00:00:00
  - 00:00:00:00:00:01
logfile: ./aeolus.log
iface: enp3s0
```
