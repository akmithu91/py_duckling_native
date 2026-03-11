FROM python:3.13-slim

# Install Rust and build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl build-essential gcc pkg-config libssl-dev ca-certificates && \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

ARG CODEARTIFACT_URL
ARG CODEARTIFACT_CARGO_URL

WORKDIR /app
COPY . .

RUN printf '[registries.codeartifact]\nindex = "sparse+%s"\ncredential-provider = "cargo:token"\n' "${CODEARTIFACT_CARGO_URL}" > /root/.cargo/config.toml

RUN --mount=type=secret,id=token \
    pip install uv && \
    export UV_INDEX_URL="${CODEARTIFACT_URL}simple/" && \
    export UV_INDEX_USERNAME=aws && \
    export UV_INDEX_PASSWORD=$(cat /run/secrets/token) && \
    export UV_EXTRA_INDEX_URL="https://pypi.org/simple" && \
    CARGO_REGISTRIES_CODEARTIFACT_TOKEN="$(cat /run/secrets/token)" uv build --wheel && \
    uv publish \
      --publish-url "${CODEARTIFACT_URL}" \
      --username aws \
      --password "$UV_INDEX_PASSWORD" \
      dist/*.whl
