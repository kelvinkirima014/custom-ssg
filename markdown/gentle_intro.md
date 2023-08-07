---
title: A Gentle Introduction to Solana.
description: Learn how to build and deploy your own programs on the fastest blockchain and the fastest growing crypto ecosystem in the world.
date: 2021-09-25T11:00:00.000Z
---

This tutorial will take you from *zero to one* in building on the [Solana network](https://solana.com/). I’ll guide you through the entire process of developing on Solana by building an on-chain program using Rust and deploying it to the Solana test net. We’ll also interact with the on-chain program using the Solana/web3js Javascript API.

You don't have to be familiar with Rust to learn from this tutorial. I’ll walk you through various Rust concepts that are necessary to understand the code and also point you to the best resources.
### Prerequisites


```rust

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


### Conclusion
Congrats! We just created a solana program, deployed it on a local cluster, and interacted with it from the client side using a JSON RPC API.  
You can use this tutorial as a reference on various Solana and Rust concepts as you build your own programs.

