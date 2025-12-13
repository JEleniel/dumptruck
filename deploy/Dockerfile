FROM apache/age:latest

ENV DEBIAN_FRONTEND=noninteractive

# Install build deps for pgvector and build it
RUN apt-get update
RUN apt-get install -y --no-install-recommends build-essential git ca-certificates wget curl libssl-dev pkg-config postgresql-server-dev-17
RUN cd /tmp
RUN git clone --branch v0.8.1 https://github.com/pgvector/pgvector.git
RUN cd pgvector
RUN make
RUN make install

EXPOSE 5432

CMD ["docker-entrypoint.sh", "postgres"]
