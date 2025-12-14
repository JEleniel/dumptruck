FROM ollama/ollama:latest

ENV DEBIAN_FRONTEND=noninteractive

# Start the Ollama daemon in the background, wait for it to be ready,
# then pull the required models. The daemon must be running for pull commands.
RUN ollama serve > /dev/null 2>&1 & \
	sleep 10 && \
	ollama pull nomic-embed-text && \
	ollama pull deepseek-r1 && \
	ollama pull qwen3

EXPOSE 11434

# Start Ollama server
CMD ["ollama", "serve"]
