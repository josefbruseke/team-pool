import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TeamPool } from "../target/types/team_pool";

describe("team-pool", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.TeamPool as Program<TeamPool>;

  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const creator = provider.wallet;


  const [pool_pda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("pool"), creator.publicKey.toBuffer()],
    program.programId,
  );

  const [pool_vault_pda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("pool_vault"), pool_pda.toBuffer()],
    program.programId
  );


  it("Is initialized!", async () => {
    const tx = await program.methods.initializePool(10, 0.1, { private: {} }, "12345").accounts({
      poolPda: pool_pda,           // Nome correto baseado no Rust (deve ser pool_pda)
      poolVaultPda: pool_vault_pda, // Nome correto baseado no Rust (deve ser pool_vault_pda)
      creator: creator.publicKey, // Nome correto baseado no Rust (deve ser new_member)
      systemProgram: anchor.web3.SystemProgram.programId // Sistema de programas do Solan
    })
      .signers([])
      .rpc();

    console.log("Your transaction signature", tx);
  });

  it("Fetch account", async () => {
    const memberAccount = anchor.web3.Keypair.generate()
    const pdaAccount = await program.account.pool.fetch(pool_pda)
    console.log(JSON.stringify(pdaAccount, null, 2))

  });

  it("Add member 1", async () => {
    const memberAccount = anchor.web3.Keypair.generate();

    const airdropSignature = await provider.connection.requestAirdrop(
      memberAccount.publicKey,
      3000000000
    );
    const balance = await provider.connection.getBalance(memberAccount.publicKey)
    console.log(balance)

    const latestBlockhash = await provider.connection.getLatestBlockhash();
    await provider.connection.confirmTransaction({
      signature: airdropSignature,
      blockhash: latestBlockhash.blockhash,
      lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
    });

    const tx = await program.methods
      .joinPool("12345")
      .accounts({
        poolPda: pool_pda,
        poolVaultPda: pool_vault_pda,
        newMember: memberAccount.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId
      })
      .signers([memberAccount])
      .rpc();

    console.log("Your transaction signature", tx);

    const poolAccountData = await program.account.pool.fetch(pool_pda);

    const new_balance = await provider.connection.getBalance(memberAccount.publicKey)
    console.log(new_balance)


    console.log("Members in pool:", poolAccountData.members);
  });

  it("Close Pool", async () => {
    const new_member = anchor.web3.Keypair.generate();

    const tx = await program.methods
      .closePool()
      .accounts({
        poolPda: pool_pda,
        authority: creator.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId
      })
      .signers([new_member])
      .rpc()

    console.log("Your transaction signature", tx)

    const poolAccountData = await program.account.pool.fetch(pool_pda);
    console.log(poolAccountData)

  });

  it("Fetch account2", async () => {
    const memberAccount = anchor.web3.Keypair.generate()
    const pdaAccount = await program.account.pool.fetch(pool_pda)
    console.log(JSON.stringify(pdaAccount, null, 2))

  });

  it("Add member 3", async () => {
    const memberAccount = anchor.web3.Keypair.generate();

    const airdropSignature = await provider.connection.requestAirdrop(
      memberAccount.publicKey,
      3000000000
    );
    const balance = await provider.connection.getBalance(memberAccount.publicKey)
    console.log(balance)

    const latestBlockhash = await provider.connection.getLatestBlockhash();
    await provider.connection.confirmTransaction({
      signature: airdropSignature,
      blockhash: latestBlockhash.blockhash,
      lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
    });

    const tx = await program.methods
      .joinPool("12345")
      .accounts({
        poolPda: pool_pda,
        poolVaultPda: pool_vault_pda,
        newMember: memberAccount.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId
      })
      .signers([memberAccount])
      .rpc();

    console.log("Your transaction signature", tx);

    const poolAccountData = await program.account.pool.fetch(pool_pda);

    const new_balance = await provider.connection.getBalance(memberAccount.publicKey)
    console.log(new_balance)


    console.log("Members in pool:", poolAccountData.members);
  });

  it("Fetch account 3", async () => {
    const memberAccount = anchor.web3.Keypair.generate()
    const pdaAccount = await program.account.pool.fetch(pool_pda)
    console.log(JSON.stringify(pdaAccount, null, 2))

  });

  it("Transfer from Vault", async () => {

    const tx = await program.methods.transferFromVault()
      .accounts({
        poolPda: pool_pda,
        poolVaultPda: pool_vault_pda,
        creatorAccount: creator.publicKey,
        authority: creator.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId
      })
      .signers([])
      .rpc();

    console.log(tx)
  })
});
