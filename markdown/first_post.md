---
title: A Gentle Introduction to Solana.
description: This tutorial will teach you how to build and deploy your own programs on the fastest blockchain and the fastest growing crypto ecosystem in the world.
date: 2021-09-25T11:00:00.000Z
---

This tutorial will take you from *zero to one* in building on the [Solana network](https://solana.com/). I’ll guide you through the entire process of developing on Solana by building an on-chain program using Rust and deploying it to the Solana test net. We’ll also interact with the on-chain program using the Solana/web3js Javascript API.

You don't have to be familiar with Rust to learn from this tutorial. I’ll walk you through various Rust concepts that are necessary to understand the code and also point you to the best resources.
### Prerequisites

- Basic familiarity with a [command-line interface](https://en.wikipedia.org/wiki/Command-line_interface)
- Basic programming knowledge in any language.

### Requirements.

You'll need the following installed before we proceed:

- [node](https://nodejs.org/en/download/) version 14+
- git
- Github account
- Rust - install [from here](https://www.rust-lang.org/tools/install#:~:text=To%20start%20using%20Rust%2C%20download,see%20%22Other%20Installation%20Methods%22.), it's pretty straightforward.
- Solana tool suite - follow [this simple guide](https://docs.solana.com/cli/install-solana-cli-tools) to install. 

### Programming on Solana - Something you should know.

Before we start coding our program, we must have an overview of what's building on Solana is like. Unlike other blockchains, in Solana, smart contracts are called Programs. Solana programs are compiled to a variation of bytecode known as [Berkley Packet Filter (BPF)](https://ebpf.io/what-is-ebpf/). Solana uses BPF  because it allows [just-in-time(JIT)](https://ebpf.io/what-is-ebpf/#jit-compilation) compilation which is great for performance. 

When called, a program must be passed to something called a [BPF loader](https://docs.solana.com/developing/on-chain-programs/overview#loaders) which is responsible for loading and executing BPF programs. All programs export an entrypoint that the runtime looks up and calls when invoking a program.

Because programs are written to target the BPF loader, they can be written in any language that compiles to BPF bytecode. Solana currently supports writing programs in Rust and C/C++.

### Our Solana program.

Solana has a nice [hello-world example](https://github.com/solana-labs/example-helloworld) that shows us how to build a Rust program on Solana from scratch and interact with it using a typescript SDK. 

The example comprises of:

- An on-chain hello world program
- A client that can send a "hello" to an account and get back the number of times "hello" has been sent.

We'll leverage this example to learn how to build our programs. Open your CLI and run the following command to clone the repo.

```bash
git clone https://github.com/solana-labs/example-helloworld.git
```

then:

```bash
cd example-helloworld
```

open the project in your IDE. In the src folder, you'll find two ways to build the program. One uses the C language while the other uses Rust. Since we are building with Rust, go ahead and open the program-rust folder and ignore program-c. There is also a client folder but we'll get to it later. For now, we are interested in `lib.rs` inside the program-rust `src`. The code looks like this:

```rust
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

/// Define the type of state stored in accounts
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct GreetingAccount {
    /// number of greetings
    pub counter: u32,
}

// Declare and export the program's entrypoint
entrypoint!(process_instruction);

// Program entrypoint's implementation
pub fn process_instruction(
    program_id: &Pubkey, // Public key of the account the hello world program was loaded into
    accounts: &[AccountInfo], // The account to say hello to
    _instruction_data: &[u8], // Ignored, all helloworld instructions are hellos
) -> ProgramResult {
    msg!("Hello World Rust program entrypoint");

    // Iterate over accounts
    let accounts_iter = &mut accounts.iter();

    // Get the account to say hello to
    let account = next_account_info(accounts_iter)?;

    // The account must be owned by the program in order to modify its data
    if account.owner != program_id {
        msg!("Greeted account does not have the correct program id");
        return Err(ProgramError::IncorrectProgramId);
    }

    // Increment and store the number of times the account has been greeted
    let mut greeting_account = GreetingAccount::try_from_slice(&account.data.borrow())?;
    greeting_account.counter += 1;
    greeting_account.serialize(&mut &mut account.data.borrow_mut()[..])?;

    msg!("Greeted {} time(s)!", greeting_account.counter);

    Ok(())
}
//Minus the tests.
```

There's a lot of awesome things going on in the above code. Let's go through it line by line, as I promised. 

Rust allows us to build on code written by others using [crates](https://learning-rust.github.io/docs/d4.crates.html). A crate can contain several modules and we specify the modules we want to bring into scope. First, we bring the crates we need via a *use* declaration. 
This is our first use declaration:

```rust
use borsh::{BorshDeserialize, BorshSerialize};
```

We specify we'll need  `BorshDeserialize` **and `BorshSerialize` from the crate `borsh`  The double colon`::`   is the path separator. Borsh is a binary serialization format. It is designed to serialize any objects to canonical and deterministic set of bytes.  `BorsheSerialize` is used for converting data(structs, ints, enums, etc) into bytecode while `BorsheDeserialize` reconstructs the bytecode into data. Serializing is necessary because the programs must be parsed in BPF format.

The next `use` declaration brings the `solana_program` crate into the scope. This crate contains a bunch of Solana source code that we'll leverage to write on-chain programs.

```rust
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};
```

Let's discuss what each item we brought from the `solana_program` crate does:

- from *account_info* we get [`next_account_info`](https://github.com/solana-labs/solana/blob/a911ae00baa9bb7031041add14c5185c86376afb/sdk/program/src/account_info.rs#L202) which is a public function that returns the next `AccountInfo` or a `NotEnoughAccountKeys` error. `AccountInfo` is a public struct(we'll discuss structs in a few) that contains the account's information - like the Pubkey and owner. You can view the source code [here](https://github.com/solana-labs/solana/blob/a911ae00baa9bb7031041add14c5185c86376afb/sdk/program/src/account_info.rs#L10).
- In `entrypoint`, we have an `entrypoint!` [macro](https://doc.rust-lang.org/rust-by-example/macros.html) that we'll later use to call our program. Macros are a way of writing code that writes other code. They reduce the amount of code you need to write! We have different forms of macros, the *entrypoint* we just brought into scope is known as a [declarative macro](https://doc.rust-lang.org/reference/macros-by-example.html) because it allows us to define syntax extension in a declarative way.
- We then bring `ProgramResult` which also lives in the [same file](https://github.com/solana-labs/solana/blob/a911ae00baa9bb7031041add14c5185c86376afb/sdk/program/src/entrypoint.rs#L18) as the entrypoint macro. It's a [Result](https://doc.rust-lang.org/stable/book/ch09-02-recoverable-errors-with-result.html?highlight=resu#recoverable-errors-with-result) type that returns `Ok` if the program runs well or `ProgramError` if the program fails. [Result](https://doc.rust-lang.org/stable/book/ch09-02-recoverable-errors-with-result.html?highlight=resu#recoverable-errors-with-result) is an enum in Rust which is defined as having two variants, `Ok` and `Err`. We use it for error handling.
- `msg` is a macro that's used for logging in Solana. If you have programmed in Rust before, you may be used to the `println!` macro but Solana considers it computationally expensive.
- `ProgramError` allows you to implement program-specific error types and see them returned by the Solana runtime.
- Lastly, we bring in the [`Pubkey`](https://github.com/solana-labs/solana/blob/a911ae00baa9bb7031041add14c5185c86376afb/sdk/program/src/pubkey.rs#L57) struct from `pubkey`. We'll use it to pass the public keys of our accounts.

One more thing I would like you to note is that whenever we bring a crate, we must also specify so in cargo.toml like so

```rust
[dependencies]
borsh = "0.9.1"
borsh-derive = "0.9.1"
solana-program = "1.7.9"
```

[Cargo](https://doc.rust-lang.org/stable/book/ch01-03-hello-cargo.html) is Rust's package manager, like npm in JS. In that line of thought, cargo.toml is analogous to package.json.  

After the use declarations, here is what we have next.

```rust
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct GreetingAccount {
    /// number of greetings
    pub counter: u32,
}
```

`#[derive]` belongs to another group of macros known as procedural macros. Deriving tells the compiler to provide some basic implementations for some traits. Besides the serialize and deserializing traits, we also derive the `Debug` trait. In Rust, traits allow us to share behaviour across non-abstract types like structs and facilitates code reuse. They are like interfaces in other languages. `Debug` trait makes types like structs and enums printable. 

Next, we declare the `GreetingAccount` *struct* using the `pub` keyword which makes it *publicly* accessible so other programs can use it. By default, everything in Rust is private, with two exceptions: Associated items in a pub Trait and Enum variants in a pub enum. A `struct` or structure is a custom data type that allows us to package related values. Each field defined within a struct has a name and a type. `GreetingAccount` has only one field: `counter` with a type of `u32`, an unsigned(positive) 32-bit integer.

### program entrypoint.

All Solana programs must have an `entrypoint` that the runtime looks up and calls when invoking a program. The [`entrypoint!`](https://github.com/solana-labs/solana/blob/a911ae00baa9bb7031041add14c5185c86376afb/sdk/program/src/entrypoint.rs#L42) macro declares `process_instruction` as the entry to our program. An instruction specifies which program it is calling, which accounts it wants to read or modify, and additional data. 

```rust
// Declare and export the program's entrypoint
entrypoint!(process_instruction);

```
We implement `process_instruction`via a function with visibility set to public.

```rust
// Program entrypoint's implementation
pub fn process_instruction(
    program_id: &Pubkey, // Public key of the account the hello world program was loaded into
    accounts: &[AccountInfo], // The account to say hello to
    _instruction_data: &[u8], // Ignored, all helloworld instructions are hellos
) -> ProgramResult {
//snip
}
```

You may have noticed that each parameter has an ampersand operator `&`. This is because  Solana programs do not store data, data is stored in accounts. The ampersand tells Rust that we do not own this data, we're just borrowing it, we call this [referencing](https://learning-rust.github.io/docs/c2.borrowing.html).

- `program_id` is the public key of the currently executing program accounts. When you want to call a program, you must also pass this id, so that Solana knows which program is to be executed. 
- `accounts` is a reference to an array of accounts to say hello to. It is the list of accounts that will be operated upon in this code.
- `_instruction_data` - any additional data passed as a u8 array. In this program, we won't be consuming this data because it's just hellos, so we add the _underscore to tell the compiler to chill.

```rust
pub fn process_instruction(
    //params
) -> ProgramResult {
    msg!("Hello World Rust program entrypoint");

    // Iterating accounts is safer then indexing
    let accounts_iter = &mut accounts.iter();

    // Get the account to say hello to
    let account = next_account_info(accounts_iter)?;

    // snip
}
```

The function returns `ProgramResult` which we imported earlier. ProgramResult is of Result type which is an Enum with two variants: Ok representing success and containing a value, and Err representing error and containing an error value. ProgramResult will give as an Ok() as a success if our *instruction* is processed or a ProgramError if it fails.

We use the [msg!](https://docs.solana.com/developing/on-chain-programs/developing-rust#logging) macro for printing messages on the program log.

We create a new variable `accounts_iter` using the let keyword. We [iterate](https://doc.rust-lang.org/stable/book/ch13-02-iterators.html?highlight=ite#processing-a-series-of-items-with-iterators) over each account using the `iter()` method and bind them to the variable as *mutable references*. Rust [references](https://learning-rust.github.io/docs/c2.borrowing.html) are immutable by default so we have to specify that we want to be able to write to each account by adding the `mut` keyword. As I mentioned, `next_account_info` will return the account we want to say hello to or an error if it doesn't find an account. It's able to do this because the function returns the `Result` type we talked of earlier. The [question mark operator](https://doc.rust-lang.org/std/result/#the-question-mark-operator-) `?` hides some of the boilerplate of propagating errors.

Only the program that owns the account should be able to modify its data. This check ensures that if the `account.owner` public key does not equal the `program_id` we will return an `IncorrectProgramId` error.

```rust
// The account must be owned by the program in order to modify its data
    if account.owner != program_id {
        msg!("Greeted account does not have the correct program id");
        return Err(ProgramError::IncorrectProgramId);
    }

```

Lastly, here's what we have...

```rust
   // Increment and store the number of times the account has been greeted
    let mut greeting_account = GreetingAccount::try_from_slice(&account.data.borrow())?;
    greeting_account.counter += 1;
    greeting_account.serialize(&mut &mut account.data.borrow_mut()[..])?;

    msg!("Greeted {} time(s)!", greeting_account.counter);

    Ok(())
```

Rust variables are immutable by default, even when declared with the `let` keyword. Therefore, to create a variable that we'll modify, we have to add the mut keyword, just like we did with references. try_from_slice() is a method from the borsh crate that we use to deserialize an instance from slice of bytes to actual data our program can work with. Under the hood, it looks like this:
```rust
fn try_from_slice(v: &[u8]) -> Result<Self>
```
try_from_slice could also return an error if the deserialization fails - note the `?` operator because it implements the *Result* type. We use the actual account data we borrowed to get the counter value and increment it by one and send it back to the runtime in serialized format.
We then print in the Program Log how many times the count has been incremented by using the msg!() macro.

### Configuring Solana CLI

First make sure you have solana installed: 
```bash
solana --version
solana-cli 1.7.11 (src:bdb77b0c; feat:1140394761)
```

In Solana, a set of validators make up a cluster. We've three clusters: mainnet, devnet and localhost. For our purposes, we'll use the local cluster.
Let's set the CLI config to the localhost cluster using the *config set* command.

```bash
solana config set --url localhost
```
The output should resemble this..

```bash
solana config set --url localhost
Config File: /home/kelvin/.config/solana/cli/config.yml
RPC URL: http://localhost:8899 
WebSocket URL: ws://localhost:8900/ (computed)
Keypair Path: /home/kelvin/.config/solana/id.json 
Commitment: confirmed 
```
1. Create CLI Keypair

If this is your first time using the Solana CLI, you will need to generate a new keypair:

```bash
solana-keygen new
```
This is the expected output
```bash
Generating a new keypair

For added security, enter a BIP39 passphrase

NOTE! This passphrase improves security of the recovery seed phrase NOT the
keypair file itself, which is stored as insecure plain text

BIP39 Passphrase (empty for none): 

Wrote new keypair to /home/kelvin/.config/solana/id.json
============================================================================
pubkey: 2ab2mQwRzTYCoXThK4mi8M7fTfGzV48ftpE3xNJvKem3
============================================================================
Save this seed phrase and your BIP39 passphrase to recover your new keypair:
make museum conduct seven dose glide recipe bring film differ excite chapter
============================================================================
```
Note that you should **never** publish your seed phrase to the internet. Am doing this for educational purposes only and will never use this keypair again.

### **Start local Solana cluster**

This example connects to a local Solana cluster by default.

Start a local Solana cluster:

```bash
solana-test-validator
```
Expected output:
```bash
solana-test-validator
Ledger location: test-ledger
Log: test-ledger/validator.log
Identity: GZr7zHFUxA7kjGgzUsUuRfQtNASBCGurynEg7yUDcfvP
Genesis Hash: F945qQyeHDUXN58eUWuLHLogAZ7Qgkpucc7xe8LisQnR
Version: 1.6.9
Shred Version: 54687
Gossip Address: 127.0.0.1:1025
TPU Address: 127.0.0.1:1027
JSON RPC URL: http://127.0.0.1:8899
⠒ 00:00:08 | Processed Slot: 16 | Confirmed Slot: 16 | Finalized Slot: 0 | Snapshot Slot:
```
And the log monitor in another terminal:
```bash
$ solana logs
Streaming transaction logs. Confirmed commitment
```
We don't see any logs yet because we have not deployed our program.
### **Build the on-chain program**

Open a third terminal to build our Rust version of the onchain program:

```bash
npm run build:program-rust
```
If the build is successful, you'll get a message informing that you should now deploy your program.
```bash
To deploy this program:
  $ solana program deploy /dist/program/helloworld.so
```

### **Deploy the on-chain program**

```bash
solana program deploy dist/program/helloworld.so
``` 
You should see something like this:
```bash
solana program deploy dist/program/helloworld.so
Program Id: CixWRTY8QcWP6F2maA9uhPLcJuch7njckQswwP5dTx9z

```
Now if you go back to the the log terminal, you should see a stream of transaction logs.

That's it with the onchain program. Let's see how to interact with it and send transactions from the clientside!

## Building the clientside.

To interact with a Solana node inside a JavaScript application, we use the solana-web3.js library, which gives a convenient interface for the RPC methods.

Now that we have an onchain program, let's see how we can make calls to the blockchain. In the helloworld program folder, run the following command to install the dependencies the client needs

```bash
cd helloworld-example
npm install
```

 The client has three typescript files: `hello_world.ts`, `main.ts`, `utils.ts`. We define our functions in *hello_world.ts* and export them to *main.ts*, which is the client's entrypoint while *utils.ts* is mostly for configurations. Here's how utils.ts looks like, without the imports.

```js
async function getConfig(): Promise<any> {
  // Path to Solana CLI config file
  const CONFIG_FILE_PATH = path.resolve(
    os.homedir(),
    '.config',
    'solana',
    'cli',
    'config.yml',
  );
  const configYml = await fs.readFile(CONFIG_FILE_PATH, {encoding: 'utf8'});
  return yaml.parse(configYml);
}
//Load and parse the Solana CLI config file to determine which RPC url to use
export async function getRpcUrl(): Promise<string> {
  try {
    const config = await getConfig();
    if (!config.json_rpc_url) throw new Error('Missing RPC URL');
    return config.json_rpc_url;
  } catch (err) {
    console.warn(
      'Failed to read RPC url from CLI config file, falling back to localhost',
    );
    return 'http://localhost:8899';
  }
}
 //Load and parse the Solana CLI config file to determine which payer to use
export async function getPayer(): Promise<Keypair> {
  try {
    const config = await getConfig();
    if (!config.keypair_path) throw new Error('Missing keypair path');
    return await createKeypairFromFile(config.keypair_path);
  } catch (err) {
    console.warn(
      'Failed to create keypair from CLI config file, falling back to new random keypair',
    );
    return Keypair.generate();
  }
}
 // Create a Keypair from a secret key stored in file as bytes' array
export async function createKeypairFromFile(
  filePath: string,
): Promise<Keypair> {
  const secretKeyString = await fs.readFile(filePath, {encoding: 'utf8'});
  const secretKey = Uint8Array.from(JSON.parse(secretKeyString));
  return Keypair.fromSecretKey(secretKey);
}
```
The `getConfig` function returns the path to the Solana CLI configuration file. In `getRpcUrl`, we determine which RPC we're using, and since we're on the local cluster, we return our localhost URL, which is Solana's default 8899.

Whenever you are sending transactions to Solana asking it to execute your instructions, like saying hello to another account - you must pay some lamports for execution. Lamport refers to the smallest denominational unit of Sol tokens(like wei in ethereum).
If the lamports are in your account, you need to sign the transaction with your private key so no one else can spend your lamports. This private key is stored in your local filesystem as an array of bytes. The `createKeypairFromFile` function decodes this array and returns it as a Keypair using the `fromSecretKey` method [provided](https://solana-labs.github.io/solana-web3.js/classes/Keypair.html#fromSecretKey) to us by the JSON rpc API. The `getPayer` function returns a Keypair that is debited everytime we make a transaction.

Now that we have our configurations in place, let's look at the code in hello_world.ts which is where the juicier stuff is. I have not included the imports and some declarations because the comments in the code are sufficient explanations.


```js
// The state of a greeting account managed by the hello world program
class GreetingAccount {
  counter = 0;
  constructor(fields: {counter: number} | undefined = undefined) {
    if (fields) {
      this.counter = fields.counter;
    }
  }
}
//Borsh schema definition for greeting accounts
const GreetingSchema = new Map([
  [GreetingAccount, {kind: 'struct', fields: [['counter', 'u32']]}],
]);
//The expected size of each greeting account.
const GREETING_SIZE = borsh.serialize(
  GreetingSchema,
  new GreetingAccount(),
).length;
```
We create a typescript class that we'll use to represent our account's data. Remember the struct we created in Rust?
```Rust
pub struct GreetingAccount {
    pub counter: u32,
}
```
The GreetingSchema constant maps the GreetingAccount from the client side to the struct in our Rust program. `kind` tell the schema that we are mapping GreetingAccount to a type struct. `fields` refers to to the name of the elements in the struct and it's type.We need to pass fields as an array because we could have multiple elements.
We serialize the data in GreetingAccount into an array of bytes and calculate its size using `.length`. We store the data in `GREETING_SIZE` and we'll later use it to calculate the rent amount we have to pay for storing data on the blockchain.(more of this in a few).

#### Establish a connection to the cluster.
```js
//Establish a connection to the cluster
export async function establishConnection(): Promise<void> {
  const rpcUrl = await getRpcUrl();
  connection = new Connection(rpcUrl, 'confirmed');
  const version = await connection.getVersion();
  console.log('Connection to cluster established:', rpcUrl, version);
}
```
Above, we establish a connection to the cluster using the establishConnection() function. Notice how we get the rpc url from the `getRpcUrl()` function we created in util.js?

#### Paying for rent and transactions.
In `utils.js`, we established an account keypair, `getPayer` that is debited every time we make a transaction. In Solana, we also have [to pay rent](https://docs.solana.com/developing/programming-model/accounts#rent) for the storage cost of keeping the account alive. However, an account can be made entirely exempt from rent collection by depositing at least 2 years worth of rent. The `getMinimumBalanceForRentExemption` API [can be used](https://docs.solana.com/developing/clients/jsonrpc-api#getminimumbalanceforrentexemption) to get the minimum balance required for a particular account. Notice how we pass the GREETING_SIZE constant we declared earlier? The RPC also [provides us](https://docs.solana.com/developing/clients/jsonrpc-api#getrecentblockhash) with the `getRecentBlockhash` function that returns a fee schedule that can be used to compute the cost of submitting a transaction. 
Validators charge a [lamportsPerSignature](https://docs.solana.com/implemented-proposals/transaction-fees#congestion-driven-fees) fee incase the network is congested. Because we are on a testnet and don't care about money, we multiply the fee by 1OO to make sure our transactions never get rejected, at least not for lack of money.
```js
//Establish an account to pay for everything
export async function establishPayer(): Promise<void> {
  let fees = 0;
  if (!payer) {
    const {feeCalculator} = await connection.getRecentBlockhash();
    // Calculate the cost to fund the greeter account
    fees += await connection.getMinimumBalanceForRentExemption(GREETING_SIZE);
    // Calculate the cost of sending transactions
    fees += feeCalculator.lamportsPerSignature * 100; // wag
    payer = await getPayer();
  }
  let lamports = await connection.getBalance(payer.publicKey);
  if (lamports < fees) {
    // If current balance is not enough to pay for fees, request an airdrop
    const sig = await connection.requestAirdrop(
      payer.publicKey,
      fees - lamports,
    );
    await connection.confirmTransaction(sig);
    lamports = await connection.getBalance(payer.publicKey);
  }

  console.log(
    'Using account',
    payer.publicKey.toBase58(),
    'containing',
    lamports / LAMPORTS_PER_SOL,
    'SOL to pay for fees',
  );
}
```
#### Check if the hello world BPF program has been deployed
The client loads the keypair of the deployed program from the file whose path we defined in `PROGRAM_KEYPAIR_PATH `constant and then read the programId from file. If the program isn't found, we return an error.

```js
export async function checkProgram(): Promise<void> {
  // Read program id from keypair file
  try {
    const programKeypair = await createKeypairFromFile(PROGRAM_KEYPAIR_PATH);
    programId = programKeypair.publicKey;
  } catch (err) {
    const errMsg = (err as Error).message;
    throw new Error(
      `Failed to read program keypair at '${PROGRAM_KEYPAIR_PATH}' due to error: ${errMsg}. Program may need to be deployed with \`solana program deploy rogram/helloworld.so\``,
    );
  }
  ```
Below, we use the `getAccountInfo` method [from the API](https://docs.solana.com/developing/clients/jsonrpc-api#getaccountinfo) to retrieve programId. We perform the following checks.
- If the programId is not found, check to see if there is a compiled binary in the filesystem. Incase there is a compiled binary, we throw an error asking the user to deploy the program. 
- If a binary is not found, we throw an error asking the user to build and deploy the program.
- Lastly, we check to see if the account is [executable](https://docs.solana.com/developing/programming-model/accounts#executable). An executable account is one that has been successfully deployed and is owned by the BPF loader.
- If all the checks are successful, we log the programId as in the console in string format.
  ```js
  // Check if the program has been deployed
  const programInfo = await connection.getAccountInfo(programId);
  if (programInfo === null) {
    if (fs.existsSync(PROGRAM_SO_PATH)) {
      throw new Error(
        'Program needs to be deployed with `solana program deploy dist/program/helloworld.so`',
      );
    } else {
      throw new Error('Program needs to be built and deployed');
    }
  } else if (!programInfo.executable) {
    throw new Error(`Program is not executable`);
  }
  console.log(`Using program ${programId.toBase58()}`);
  ```
Using the `createWithSeed` method from [web3js](https://solana-labs.github.io/solana-web3.js/classes/PublicKey.html#createWithSeed), we derive a public key from another key, a seed, and a program ID. The program ID will also serve as the owner of the public key, giving it permission to write data to the account.
We check to see if the account does not already exist, and if so, make a transaction to create the account using the SystemProgram's createAccountFromSeed property. In Solana, the system program is responsible for creating new accounts.
`sendAndConfirmTransaction` [does what it says](https://solana-labs.github.io/solana-web3.js/modules.html#sendAndConfirmTransaction). We have to pass the RPC endpoint, transaction we just created, and the signer as parameters.
  ```js
  // Derive the address (public key) of a greeting account from the program so that it's easy to find later.
  const GREETING_SEED = 'hello';
  greetedPubkey = await PublicKey.createWithSeed(
    payer.publicKey,
    GREETING_SEED,
    programId,
  );
// Check if the greeting account has already been created
  const greetedAccount = await connection.getAccountInfo(greetedPubkey);
  if (greetedAccount === null) {
    console.log(
      'Creating account',
      greetedPubkey.toBase58(),
      'to say hello to',
    );
    const lamports = await connection.getMinimumBalanceForRentExemption(
      GREETING_SIZE,
    );

    const transaction = new Transaction().add(
      SystemProgram.createAccountWithSeed({
        fromPubkey: payer.publicKey,
        basePubkey: payer.publicKey,
        seed: GREETING_SEED,
        newAccountPubkey: greetedPubkey,
        lamports,
        space: GREETING_SIZE,
        programId,
      }),
    );
    await sendAndConfirmTransaction(connection, transaction, [payer]);
  }
}
```
In the sayHello function below, we create an instruction using the `TransactionInstruction` [class from](https://solana-labs.github.io/solana-web3.js/classes/TransactionInstruction.html) web3 API. The keys is the account metadata which takes the following format:
`AccountMeta: { pubkey: PublicKey; isSigner: boolean; isWritable: boolean; }`
In our example, the pubkey we pass as metadata is that of the greetedAccount we're saying hello to. We also state that the transaction doesn't need a signer and it's both read and write.
We allocate 0 bytes data size because we didn't pass any data while creating our program. We said it just *hellos*, remember?
We send and confirm just like we did in the other transaction.
```js
// Say hello
export async function sayHello(): Promise<void> {
  console.log('Saying hello to', greetedPubkey.toBase58());
  const instruction = new TransactionInstruction({
    keys: [{pubkey: greetedPubkey, isSigner: false, isWritable: true}],
    programId,
    data: Buffer.alloc(0), // All instructions are hellos
  });
  await sendAndConfirmTransaction(
    connection,
    new Transaction().add(instruction),
    [payer],
  );
}
```
Every time a client says hello, `GreetingAccount `increments counter by one. We deserialize GreetingAccount to get how many times the account we've created has been greeted from the counter and log the number to the console.
```js
// Report the number of times the greeted account has been said hello to
export async function reportGreetings(): Promise<void> {
  const accountInfo = await connection.getAccountInfo(greetedPubkey);
  if (accountInfo === null) {
    throw 'Error: cannot find the greeted account';
  }
  const greeting = borsh.deserialize(
    GreetingSchema,
    GreetingAccount,
    accountInfo.data,
  );
  console.log(
    greetedPubkey.toBase58(),
    'has been greeted',
    greeting.counter,
    'time(s)',
  );
}
```
If you go ahead and run `npm start`, you should have the following output:
```bash
Let's say hello to a Solana account...
Connection to cluster established: http://localhost:8899 { 'feature-set': 1140394761, 'solana-core': '1.7.11' }
Using account 2ab2mQwRzTYCoXThK4mi8M7fTfGzV48ftpE3xNJvKem3 containing 499999999.1441591 SOL to pay for fees
Using program CixWRTY8QcWP6F2maA9uhPLcJuch7njckQswwP5dTx9z
Creating account 9wtyFcYpzTnP6JgJow7KtFn7KbRymEfq5L2uqvEvq9cS to say hello to
Saying hello to 9wtyFcYpzTnP6JgJow7KtFn7KbRymEfq5L2uqvEvq9cS
9wtyFcYpzTnP6JgJow7KtFn7KbRymEfq5L2uqvEvq9cS has been greeted 1 time(s)
Success
```
Run npm start again and you'll see the times we say hello increases by 1 each time. Ever seen a hello world tutorial longer this before?

### Conclusion
Congrats! We just created a solana program, deployed it on a local cluster, and interacted with it from the client side using a JSON RPC API.  
You can use this tutorial as a reference on various Solana and Rust concepts as you build your own programs.

