# Simple Event-Driven Microservice Architecture in Rust

## Overview
"Simple Event-Driven Microservice Architecture in Rust" is an open-source project showcasing the simplicity and efficiency of building microservices in Rust. This minimalistic project demonstrates an e-commerce backend system, featuring just two microservices: Catalog and Order.

## Getting Started

### Prerequisites
During the installation steps you will install the following dependencies when running install_prerequisites_macos.sh
- Rust
- Brew
- Java
- Kafka

### Installation Steps (MacOs)

1. **Set Executable Permissions**  
   Open your terminal at the root of the project and run the following commands:
   ```bash
   chmod +x .scripts/install_prerequisites_macos.sh
   chmod +x pre_run.sh

2. **Install Dependencies**  
   Execute the script to install prerequisites:
   ```bash
   ./.scripts/install_prerequisites_macos.sh

3. **Run Kafka Server**  
   Start the Kafka server, which is used for event passing between microservices:
   ```bash
   ./pre_run.sh
   
4. **Start Microservices**  
   Run both the Catalog and Order microservices in separate terminal windows:
- For Catalog Microservice:
  ```
  (cd ./catalog_service && cargo run)
  ```
- For Order Microservice:
  ```
  (cd ./order_service && cargo run)
  ```

## Usage

Once the server is operational, you can interact with the microservices through the following endpoints:

- **Catalog Microservice:**  
  `GET http://127.0.0.1:8081/catalog`  
  Retrieves a JSON list of all available products.

- **Order Microservice:**  
  `POST http://127.0.0.1:8080/order`  
  Creates an order in the system. Use the header `Content-Type: application/json` and the following JSON body structure:
   ```json
  {
  "item_id": 1,
  "name": "James",
  "address": "22 Bugs Bunny Street, London, E1 4AH, United Kingdom",
  "quantity": 1
  }

## Contributing
Contributions are welcome! Please feel free to submit pull requests or open issues for improvements and suggestions.
Prior to contributing please ensure you read CONTRIBUTING.md.

## License
This project is open source and available under MIT.

#### Happy Coding! 🚀🦀

