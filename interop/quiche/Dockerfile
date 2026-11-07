FROM ubuntu:24.04 AS build
ARG QUICHE_COMMIT=854314dcc99028933a436ebe29716e084ffa4525
ARG BAZELISK_VERSION=v1.25.0
RUN apt-get update && apt-get install -y --no-install-recommends \
      build-essential clang git curl ca-certificates python3 libicu-dev \
 && rm -rf /var/lib/apt/lists/*
RUN curl -fsSL -o /usr/local/bin/bazel \
      https://github.com/bazelbuild/bazelisk/releases/download/${BAZELISK_VERSION}/bazelisk-linux-$(dpkg --print-architecture) \
 && chmod +x /usr/local/bin/bazel
WORKDIR /src
RUN git init -q && git remote add origin https://github.com/google/quiche \
 && git fetch -q --depth 1 origin ${QUICHE_COMMIT} && git checkout -q FETCH_HEAD
ENV CC=clang CXX=clang++
RUN bazel build -c opt --jobs=4 --local_resources=memory=3000 \
      //quiche:masque_server //quiche:masque_client \
 && mkdir /out && cp bazel-bin/quiche/masque_server bazel-bin/quiche/masque_client /out/

FROM ubuntu:24.04
RUN apt-get update && apt-get install -y --no-install-recommends openssl ca-certificates libicu74 \
 && rm -rf /var/lib/apt/lists/*
COPY --from=build /out/ /usr/local/bin/
