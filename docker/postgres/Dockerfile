FROM apache/age:latest

ENV DEBIAN_FRONTEND=noninteractive

# Install build dependencies and compile pgvector
RUN apt-get update && \
	apt-get install -y --no-install-recommends \
	build-essential \
	git \
	ca-certificates \
	wget \
	curl \
	libssl-dev \
	pkg-config \
	postgresql-server-dev-17 && \
	git clone --branch v0.8.1 https://github.com/pgvector/pgvector.git /tmp/pgvector && \
	cd /tmp/pgvector && \
	make && \
	make install && \
	cd / && \
	rm -rf /tmp/pgvector && \
	apt-get remove -y build-essential git ca-certificates wget && \
	apt-get clean && \
	rm -rf /var/lib/apt/lists/*

EXPOSE 5432

CMD ["docker-entrypoint.sh", "postgres"]
