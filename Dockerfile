FROM python:3.13-slim

# Install Rust and build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl build-essential gcc pkg-config libssl-dev ca-certificates && \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

ARG CODEARTIFACT_URL
ARG CODEARTIFACT_CARGO_URL
ARG CODEARTIFACT_TOKEN
ENV CARGO_REGISTRIES_CODEARTIFACT_TOKEN="Bearer ${CODEARTIFACT_TOKEN}"

WORKDIR /app
COPY . .

RUN printf '[registries.codeartifact]\nindex = "sparse+%s"\ncredential-provider = "cargo:token"\n' "${CODEARTIFACT_CARGO_URL}" > /root/.cargo/config.toml && \
    echo "DEBUG: CARGO token length=${#CARGO_REGISTRIES_CODEARTIFACT_TOKEN}" && \
    curl -s -o /dev/null -w "DEBUG: cargo endpoint HTTP status=%{http_code}\n" \
      -H "Authorization: ${CARGO_REGISTRIES_CODEARTIFACT_TOKEN}" \
      "${CODEARTIFACT_CARGO_URL}config.json"

RUN --mount=type=secret,id=token \
    TOKEN=$(cat /run/secrets/token) && \
    echo "--- DEBUG auth tests ---" && \
    curl -s -o /dev/null -w "PyPI  Basic  auth: %{http_code}\n" -u "aws:$TOKEN" "${CODEARTIFACT_URL}simple/" && \
    curl -s -o /dev/null -w "Cargo Bearer auth: %{http_code}\n" -H "Authorization: Bearer $TOKEN" "${CODEARTIFACT_CARGO_URL}config.json" && \
    curl -s -o /dev/null -w "Cargo Basic  auth: %{http_code}\n" -u "aws:$TOKEN" "${CODEARTIFACT_CARGO_URL}config.json" && \
    echo "--- END DEBUG ---" && \
    pip install uv && \
    export UV_INDEX_URL="${CODEARTIFACT_URL}simple/" && \
    export UV_INDEX_USERNAME=aws && \
    export UV_INDEX_PASSWORD=$(cat /run/secrets/token) && \
    export UV_EXTRA_INDEX_URL="https://pypi.org/simple" && \
    uv build --wheel && \
    uv publish \
      --publish-url "${CODEARTIFACT_URL}" \
      --username aws \
      --password "$UV_INDEX_PASSWORD" \
      dist/*.whl
