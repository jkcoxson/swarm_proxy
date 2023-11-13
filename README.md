# Swarm Proxy

A proxy that forwards a lot of ports really fast
(as in starting and config, idk about perf)

## Motivation

I had a server behind a double NAT, and masquerading all the ports would have been a lot of effort.
I'm lazy, so I wrote a proxy to work around a solvable problem :)

## Usage

### Command line

```bash
swarm_proxy <target> udp [<udp_port|udp_range>] tcp [<tcp_port|tcp_range>]
```

#### Example

Forwards ports 69-79 udp and 80 and 443 tcp to 10.7.0.2

```bash
swarm_proxy 10.7.0.2 udp 69:79 tcp 80 443
```

## Design

### TCP

Super simple, connection goes in, new connection made and packets are exchanged.

### UDP

Not as simple. UDP is stateless, but server needs to know which port to forward to on the way back.
When a packet is sent, a new port is bound in the proxy server only for sending back to that one client.
Clients are bound for about five minutes, then the port is released. Tokio channels go brrrr.

## Speed

Pretty quick, I'm not about to talk trash about my own proxy.
