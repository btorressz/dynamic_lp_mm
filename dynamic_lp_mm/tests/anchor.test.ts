// No imports needed: web3, pg, anchor are all globally available
//Basic test file

describe("Dynamic LP Vault Full Test", () => {
  let vaultPda: web3.PublicKey;
  let shareMintPda: web3.PublicKey;
  let baseVaultPda: web3.PublicKey;
  let quoteVaultPda: web3.PublicKey;
  let vaultBump: number;

  const baseMint = new web3.Keypair();
  const quoteMint = new web3.Keypair();
  const treasury = new web3.Keypair();

  it("initialize vault", async () => {
    [vaultPda, vaultBump] = await web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), baseMint.publicKey.toBuffer(), quoteMint.publicKey.toBuffer()],
      pg.program.programId
    );

    [shareMintPda] = await web3.PublicKey.findProgramAddressSync(
      [Buffer.from("share_mint"), vaultPda.toBuffer()],
      pg.program.programId
    );

    [baseVaultPda] = await web3.PublicKey.findProgramAddressSync(
      [Buffer.from("base_vault"), vaultPda.toBuffer()],
      pg.program.programId
    );

    [quoteVaultPda] = await web3.PublicKey.findProgramAddressSync(
      [Buffer.from("quote_vault"), vaultPda.toBuffer()],
      pg.program.programId
    );

    await pg.program.methods
      .initialize(100, 5) 
      .accounts({
        vault: vaultPda,
        authority: pg.wallet.publicKey,
        treasury: treasury.publicKey,
        baseMint: baseMint.publicKey,
        quoteMint: quoteMint.publicKey,
        shareMint: shareMintPda,
        baseVault: baseVaultPda,
        quoteVault: quoteVaultPda,
        tokenProgram: web3.SystemProgram.programId, 
        systemProgram: web3.SystemProgram.programId,
        rent: web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc();

    console.log("✅ Vault initialized");
  });

  it("fake deposit", async () => {
    const depositAmount = new anchor.BN(1000000); // 1 token

    const fakeUserBaseAta = new web3.Keypair();
    const fakeUserQuoteAta = new web3.Keypair();
    const fakeUserShareAta = new web3.Keypair();

    const tx = await pg.program.methods
      .deposit(depositAmount, depositAmount)
      .accounts({
        vault: vaultPda,
        shareMint: shareMintPda,
        baseVault: baseVaultPda,
        quoteVault: quoteVaultPda,
        user: pg.wallet.publicKey,
        userBaseAta: fakeUserBaseAta.publicKey,
        userQuoteAta: fakeUserQuoteAta.publicKey,
        userShareAta: fakeUserShareAta.publicKey,
        tokenProgram: web3.SystemProgram.programId, 
      })
      .rpc();

    console.log("✅ Deposit Tx", tx);
  });

  it("fake withdraw", async () => {
    const withdrawAmount = new anchor.BN(1000000); // 1 share

    const fakeUserBaseAta = new web3.Keypair();
    const fakeUserQuoteAta = new web3.Keypair();
    const fakeUserShareAta = new web3.Keypair();

    const tx = await pg.program.methods
      .withdraw(withdrawAmount)
      .accounts({
        vault: vaultPda,
        shareMint: shareMintPda,
        baseVault: baseVaultPda,
        quoteVault: quoteVaultPda,
        user: pg.wallet.publicKey,
        userBaseAta: fakeUserBaseAta.publicKey,
        userQuoteAta: fakeUserQuoteAta.publicKey,
        userShareAta: fakeUserShareAta.publicKey,
        tokenProgram: web3.SystemProgram.programId,
      })
      .rpc();

    console.log("✅ Withdraw Tx", tx);
  });

  it("rebalance vault", async () => {
    const fakePrice = new anchor.BN(10000);

    const tx = await pg.program.methods
      .rebalance(fakePrice)
      .accounts({
        vault: vaultPda,
        authority: pg.wallet.publicKey,
        baseVault: baseVaultPda,
        quoteVault: quoteVaultPda,
        tokenProgram: web3.SystemProgram.programId,
        clock: web3.SYSVAR_CLOCK_PUBKEY,
      })
      .rpc();

    console.log("✅ Rebalance Tx", tx);
  });

  it("sweep fees", async () => {
    const fakeTreasuryBaseAta = new web3.Keypair();
    const fakeTreasuryQuoteAta = new web3.Keypair();

    const tx = await pg.program.methods
      .sweepFees()
      .accounts({
        vault: vaultPda,
        authority: pg.wallet.publicKey,
        baseVault: baseVaultPda,
        treasuryBaseAta: fakeTreasuryBaseAta.publicKey,
        quoteVault: quoteVaultPda,
        treasuryQuoteAta: fakeTreasuryQuoteAta.publicKey,
        tokenProgram: web3.SystemProgram.programId,
      })
      .rpc();

    console.log("✅ Sweep Fees Tx", tx);
  });
});
