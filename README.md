# kubecraft-proxy

<p>
  <a href="https://github.com/kubecraft-cloud/kubecraft-proxy/graphs/contributors">
    <img src="https://img.shields.io/github/contributors/kubecraft-cloud/kubecraft-proxy" alt="contributors" />
  </a>
  <a href="https://github.com/kubecraft-cloud/kubecraft-proxy/commits/main">
    <img src="https://img.shields.io/github/last-commit/kubecraft-cloud/kubecraft-proxy" alt="last update" />
  </a>
  <a href="https://github.com/kubecraft-cloud/kubecraft-proxy/issues/">
    <img src="https://img.shields.io/github/issues/kubecraft-cloud/kubecraft-proxy" alt="open issues" />
  </a>
  <a href="https://github.com/kubecraft-cloud/kubecraft-proxy/blob/master/LICENSE">
    <img src="https://img.shields.io/github/license/kubecraft-cloud/kubecraft-proxy.svg" alt="license" />
  </a>
</p>

A reverse proxy for Minecraft server using gRPC for configuration. The goal is to provide a simple way to configure a Minecraft server without having to restart the proxy.

This is useful for Minecraft servers running in a cloud environment where the servers are ephemeral and can be created at any time or for users that want to expose multiple servers on the same IP address and port.

## Features

- [x] Reverse proxy for Minecraft servers
- [x] gRPC API for configuration
- [x] Support for multiple Minecraft versions at the same time

## Roadmap

- Display Placeholder Server
- TCPShield/RealIP Protocol Support
- Prometheus Support

## Installation

The best way to install the proxy is to use the provided Docker image. The image is available on [Docker Hub](https://hub.docker.com/r/kubecraft/kubecraft-proxy).

```bash
docker run -d -p 25565:25565 -p 65535:65535 kubecraft/kubecraft-proxy:latest
```

> The proxy requires the following ports to be exposed:
>
> - 25565: Minecraft server port
> - 65535: gRPC server port

> Note: Please make sure to not expose the gRPC port to the public internet as it is not secured and everyone can change the configuration of the proxy.

But, if you don't want to use Docker, you can download the binary from the [releases page](https://github.com/kubecraft-cloud/kubecraft-proxy/releases) or build it yourself following the instructions below.

```bash
git clone https://github.com/kubecraft-cloud/kubecraft-proxy
cd kubecraft-proxy/
cargo build --release
```

You can then run the proxy using the following command:

```bash
./target/release/kubecraft-proxy
```

## Configuration

The proxy can be configured using the gRPC API. The API is available on port `65535` by default.

> ⚠️ The API is not secured and should not be exposed to the public internet.

### Example

The following example shows how to configure the proxy with the gRPC API, in the example we use [grpcurl](https://github.com/fullstorydev/grpcurl) to interact with the API but you can use any gRPC client you want.

You can find the protobuf definition of the API [here](https://github.com/kubecraft-cloud/kubecraft-proxy/blob/main/proto/src/proxy.proto).

#### Get all minecraft servers

This example shows how to get all the Minecraft servers in the proxy configuration.

```bash
grpcurl -plaintext localhost:65535 proxy.ProxyService/ListBackend
```

#### Put a new minecraft server

This example shows how to put a new Minecraft server in the proxy configuration. The proxy will then redirect all the traffic that matches the hostname `game.example.com` to the Minecraft server at `192.168.1.10:25565`.

```bash
grpcurl -plaintext -d '{"hostname":"game.example.com","redirect_ip":"192.168.1.10","redirect_port":25565}' \
    localhost:65535 proxy.ProxyService/PutBackend
```

#### Delete a minecraft server

This example shows how to delete a Minecraft server from the proxy configuration. The proxy will then stop redirecting all the traffic that matches the hostname `game.example.com`.

```bash
grpcurl -plaintext -d '{"hostname":"game.example.com","redirect_ip":"192.168.1.10","redirect_port":25565}' \
    localhost:65535 proxy.ProxyService/DeleteBackend
```

# Contributing

Contributions are welcome. Please follow the standard Git workflow - fork, branch, and pull request.

# License

This project is licensed under the Apache 2.0 - see the `LICENSE` file for details.
