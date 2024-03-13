# Piston

## Overview

Piston is a gRPC-based service designed for compressing and decompressing data. Utilizing the Huffman coding algorithm, it offers an efficient method for reducing data size, ideal for storage and transmission purposes. The service provides a simple yet powerful API, consisting of two primary RPC methods: `Compress` and `Decompress`.

## Service API


### RPC Methods

- **Compress**: Accepts `CompressRequest` and returns `CompressResponse`, compressing the provided data using Huffman coding.
- **Decompress**: Accepts `DecompressRequest` and returns `DecompressResponse`, reconstructing the original data from its compressed form.

### Protobuf Definitions

- **ByteMapping**: A mapping from a byte to a series of bits representing the Huffman-encoded sequence.
  
    ```proto
    message ByteMapping {
      uint32 byte = 1;
      repeated bool bits = 2;
    }
    ```

- **CompressRequest**: Contains the raw data to be compressed.
  
    ```proto
    message CompressRequest { bytes data = 1; }
    ```

- **CompressResponse**: Contains the compressed data, byte mappings for the Huffman tree, and the count of unique bytes processed.
  
    ```proto
    message CompressResponse {
      bytes data = 1;
      repeated ByteMapping byte_mappings = 2;
      uint32 count = 3;
    }
    ```

- **DecompressRequest**: Contains compressed data and byte mappings for decompression.
  
    ```proto
    message DecompressRequest {
      bytes data = 1;
      repeated ByteMapping byte_mappings = 2;
      uint32 count = 3;
    }
    ```

- **DecompressResponse**: Contains the original, decompressed data.
  
    ```proto
    message DecompressResponse { bytes data = 1; }
    ```


## Interacting with the Piston Service using gRPC UI

`grpcui` is an interactive web-based UI that provides an easy way to explore and test gRPC services. It's especially useful for services like Piston, allowing users to make RPC calls without writing a client.

### Getting Started with grpcui

1. Install `grpcui` by running:

    ```shell
    go get github.com/fullstorydev/grpcui
    go install github.com/fullstorydev/grpcui/cmd/grpcui
    ```

    Ensure your `GOPATH/bin` is in your `PATH`.

2. Start the Piston service as described in the "Running the Service" section.

3. Launch `grpcui` and point it to your running Piston service:

    ```shell
    grpcui -plaintext [your-piston-service-address]
    ```

    Replace `[your-piston-service-address]` with the address where your Piston service is running (e.g., `localhost:50051`).

4. `grpcui` will open a web interface in your default browser. Here, you can explore the service's methods, compose requests, and invoke them to see the responses.

### Using grpcui to Test Piston

- **Compress**: In the grpcui interface, select the `Compress` method. Input your raw data in the request field and submit the request. The interface will display the compressed data and the byte mappings.
  
- **Decompress**: Select the `Decompress` method. Input the previously received compressed data and byte mappings into the request fields and submit. The original data will be reconstructed and displayed.

`grpcui` provides an effective platform for testing and interacting with the Piston service, facilitating a deeper understanding of its operations and potential use cases.
