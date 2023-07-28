# Token Sale Contract

This is a basic token sale contract that allows users to buy tokens at a specified price. The sale is batched, meaning users can participate in batches with specific durations and amounts. The contract supports updating the configuration, starting and stopping the sale, and allowing users to claim their tokens after a distribution phase.

## Contract Functions

### Instantiate

The `instantiate` function is the contract's initialization entry point. It sets up the initial configuration and state of the contract. The admin of the contract is set as the sender if not specified in the `InstantiateMsg`.

### Execute

The `execute` function is the main entry point for handling contract execution messages. It supports the following messages:

- `Buy`: Allows users to buy tokens by sending the required payment in the specified denomination.
- `UpdateConfig`: Allows the contract admin to update the contract configuration, including batch duration, batch amount, price, and other parameters.
- `StartSale`: Allows the contract admin to start the token sale phase after configuring the contract.
- `StartDistribution`: Allows the contract admin to start the token distribution phase after the sale phase.
- `AdminWithdraw`: Allows the contract admin to withdraw unsold tokens during the distribution phase.
- `Claim`: Allows users to claim their allocated tokens after the distribution phase.

### Query

The `query` function is used to query contract information. It supports the following queries:

- `QueryConfig`: Retrieves the current contract configuration, including batch details and other parameters.
- `QueryState`: Retrieves the current contract state, including the sale status, total tokens sold, and total revenue generated.
- `QueryPosition`: Retrieves a user's position in the contract, including the total tokens bought, total tokens paid, total tokens claimed, and batch information.

## Contract Features

- Users can participate in the token sale by buying tokens at a specified price.
- Distribution is divided into batches.
- The contract allows the admin to configure various parameters, such as batch duration, batch amount, and the sale price.
- The sale can be started and stopped by the contract admin.
- After the sale phase, users can claim their allocated tokens during the distribution phase.
- The contract supports CW20 tokens and can handle various denominations.

## Contract Usage

1. Instantiate the contract with the desired configuration using the `InstantiateMsg` with the required parameters.
2. The admin can update the contract configuration using the `UpdateConfig` message before starting the sale.
3. Start the sale using the `StartSale` message to allow users to buy tokens.
4. After the sale phase is complete, start the distribution phase using the `StartDistribution` message.
5. Users can claim their allocated tokens during the distribution phase using the `Claim` message.
6. The contract admin can withdraw any unsold tokens during the distribution phase using the `AdminWithdraw` message.
7. Users can query the contract configuration, state, and their position using the corresponding query messages.

## Notes
- The contract should be carefully tested and audited for security vulnerabilities before deployment in production.
