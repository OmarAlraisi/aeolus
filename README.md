# Aeolus

<h6>Ancient Greek: Αἴολος, Aiolos - In Greek mythology, Aeolus the son of Hippotes, was the ruler of the winds.</h6>

This is a proof-of-concept (PoC) implementation of Unimog.

## Usage:

```
Usage: aeolus [OPTIONS]

Options:
  -s, --servers <IP>  Comma separated servers
  -p, --ports <PORT>              Comma separated ports [default: 80]
  -i, --iface <NI>                Netowrk interface of the virtual IP [default: wlp1s0]
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
  - 192.168.100.1
  - 192.168.100.2
  - 192.168.100.3
logfile: ./aeolus.log
```
