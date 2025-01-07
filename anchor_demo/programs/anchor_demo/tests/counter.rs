use anchor_lang::{
    prelude::Pubkey, system_program, AccountDeserialize, InstructionData, ToAccountMetas,
};
use solana_program_test::{tokio, ProgramTest, ProgramTestContext};
use solana_sdk::{
    account::Account, instruction::Instruction, signature::Keypair, signer::Signer,
    transaction::Transaction,
};

#[tokio::test]
async fn test_initialize() {
    let owner = Keypair::new();
    let program_id = anchor_demo::ID;

    let mut program_test = ProgramTest::new("anchor_demo", program_id, None);

    program_test.add_account(
        owner.pubkey(),
        Account {
            lamports: 1_000_000_000,
            ..Account::default()
        },
    );

    let (counter_pda, _) = Pubkey::find_program_address(&[b"counter"], &program_id);

    let mut context = program_test.start_with_context().await;

    let init_ix = Instruction {
        program_id: anchor_demo::ID,
        accounts: anchor_demo::accounts::Initialize {
            counter: counter_pda,
            owner: owner.pubkey(),
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: anchor_demo::instruction::Initialize {}.data(),
    };

    let init_tx = Transaction::new_signed_with_payer(
        &[init_ix],
        Some(&owner.pubkey()),
        &[&owner],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction(init_tx)
        .await
        .unwrap();

    let counter: anchor_demo::Counter = load_and_deserialize(context, counter_pda).await;

    assert_eq!(counter.count, 0);
}

#[tokio::test]
async fn test_increment() {
    let owner = Keypair::new();
    let program_id = anchor_demo::ID;

    let mut program_test = ProgramTest::new("anchor_demo", program_id, None);

    program_test.add_account(
        owner.pubkey(),
        Account {
            lamports: 1_000_000_000,
            ..Account::default()
        },
    );

    let (counter_pda, _) = Pubkey::find_program_address(&[b"counter"], &program_id);

    let mut context = program_test.start_with_context().await;

    let init_ix = Instruction {
        program_id: anchor_demo::ID,
        accounts: anchor_demo::accounts::Initialize {
            counter: counter_pda,
            owner: context.payer.pubkey(),
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: anchor_demo::instruction::Initialize {}.data(),
    };

    let increment_ix = Instruction {
        program_id: anchor_demo::ID,
        accounts: anchor_demo::accounts::MathOperation {
            counter: counter_pda,
        }
        .to_account_metas(None),
        data: anchor_demo::instruction::MathOperation {
            operation: anchor_demo::Operation::Add(1),
        }
        .data(),
    };

    let init_increment_tx = Transaction::new_signed_with_payer(
        &[init_ix, increment_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    let _res = context
        .banks_client
        .process_transaction(init_increment_tx)
        .await;

    let counter: anchor_demo::Counter = load_and_deserialize(context, counter_pda).await;

    assert_eq!(counter.count, 1);
}

#[tokio::test]
async fn test_decrement() {
    let owner = Keypair::new();
    let program_id = anchor_demo::ID;

    let mut program_test = ProgramTest::new("anchor_demo", program_id, None);

    program_test.add_account(
        owner.pubkey(),
        Account {
            lamports: 1_000_000_000,
            ..Account::default()
        },
    );

    let (counter_pda, _) = Pubkey::find_program_address(&[b"counter"], &program_id);

    let mut context = program_test.start_with_context().await;

    let init_ix = Instruction {
        program_id: anchor_demo::ID,
        accounts: anchor_demo::accounts::Initialize {
            counter: counter_pda,
            owner: context.payer.pubkey(),
            system_program: system_program::ID,
        }
        .to_account_metas(None),
        data: anchor_demo::instruction::Initialize {}.data(),
    };

    let increment_ix = Instruction {
        program_id: anchor_demo::ID,
        accounts: anchor_demo::accounts::MathOperation {
            counter: counter_pda,
        }
        .to_account_metas(None),
        data: anchor_demo::instruction::MathOperation {
            operation: anchor_demo::Operation::Add(2),
        }
        .data(),
    };

    let decrement_ix = Instruction {
        program_id: anchor_demo::ID,
        accounts: anchor_demo::accounts::MathOperation {
            counter: counter_pda,
        }
        .to_account_metas(None),
        data: anchor_demo::instruction::MathOperation {
            operation: anchor_demo::Operation::Sub(1),
        }
        .data(),
    };

    let init_increment_tx = Transaction::new_signed_with_payer(
        &[init_ix, increment_ix, decrement_ix],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    let _res = context
        .banks_client
        .process_transaction(init_increment_tx)
        .await;

    let counter: anchor_demo::Counter = load_and_deserialize(context, counter_pda).await;

    assert_eq!(counter.count, 1);
}

pub async fn load_and_deserialize<T: AccountDeserialize>(
    mut ctx: ProgramTestContext,
    address: Pubkey,
) -> T {
    let account = ctx
        .banks_client
        .get_account(address)
        .await
        .unwrap() //unwraps the Result into an Option<Account>
        .unwrap(); //unwraps the Option<Account> into an Account

    T::try_deserialize(&mut account.data.as_slice()).unwrap()
}
