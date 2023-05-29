#This is a no-op dockerfile for local development. When deploying to devices, Dockerfile.template will be used by balena
FROM alpine

CMD ["sleep","infinity"]