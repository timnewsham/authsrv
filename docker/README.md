
# Build and Run
To build:

```docker build -t authsrv-builder -f ./docker/Dockerfile .```

To run:

```docker run -p 8000:8000 authsrv-builder```

# TODO
* The db password is hard-coded. It should be automatically generated.

