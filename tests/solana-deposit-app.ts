import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { SolanaDepositApp } from "../target/types/solana_deposit_app";
import { assert } from "chai";

describe("solana-deposit-app", () => {
    // Configure the client to use the local cluster.
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    const program = anchor.workspace.SolanaDepositApp as Program<SolanaDepositApp>;

    let user = anchor.web3.Keypair.generate();
    let pda: PublicKey;
    let bump: number;

    before(async () => {
        // Airdrop SOL to the user
        const signature = await provider.connection.requestAirdrop(user.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL);
        await provider.connection.confirmTransaction(signature);

        // Derive PDA
        [pda, bump] = await PublicKey.findProgramAddressSync(
            [Buffer.from("user_account"), user.publicKey.toBuffer()],
            program.programId
        );
    });

    it("User can deposit SOL", async () => {
        const initialPdaBalance = await provider.connection.getBalance(pda);
        assert.equal(initialPdaBalance, 0, "PDA balance should be zero before deposit");
        const depositAmount = 0.5 * anchor.web3.LAMPORTS_PER_SOL;
        const balance_before = await provider.connection.getBalance(user.publicKey);
        console.log("User balance before deposit:", balance_before / anchor.web3.LAMPORTS_PER_SOL);

        console.log("Depositing SOL...");
        await program.methods
            .deposit(new anchor.BN(depositAmount))
            .accounts({
                user: user.publicKey,
                userAccount: pda,
                systemProgram: SystemProgram.programId,
            })
            .signers([user])
            .rpc();

        const balance_after = await provider.connection.getBalance(user.publicKey);
        console.log("User balance after deposit:", balance_after / anchor.web3.LAMPORTS_PER_SOL);

        const pdaBalance = await provider.connection.getBalance(pda);
        console.assert(pdaBalance >= 0.5, "PDA balance should not be negative");
    });

    it("User can withdraw SOL", async () => {
        const withdrawAmount = 0.3 * anchor.web3.LAMPORTS_PER_SOL;

        const balance_before = await provider.connection.getBalance(user.publicKey);
        console.log("User balance before withdraw:", balance_before / anchor.web3.LAMPORTS_PER_SOL);

        await program.methods
            .withdraw(new anchor.BN(withdrawAmount))
            .accounts({
                user: user.publicKey,
                userAccount: pda,
                systemProgram: SystemProgram.programId,
            })
            .signers([user])
            .rpc();

        const balance_after = await provider.connection.getBalance(user.publicKey);
        console.log("User balance after withdraw:", balance_after / anchor.web3.LAMPORTS_PER_SOL);

        const pdaBalance = await provider.connection.getBalance(pda);
        console.assert(pdaBalance >= 0.2, "PDA balance should not be negative");
    });

    it("Can get user balance via contract", async () => {
        const balance = await program.methods
            .getUserBalance()
            .accounts({
                user: user.publicKey,
                userAccount: pda,
            })
            .view();

        const actualBalance = await provider.connection.getBalance(pda);

        assert.equal(balance.toString(), actualBalance.toString());
    });

    it("Cannot withdraw more than balance", async () => {
        try {
            const overWithdraw = 1 * anchor.web3.LAMPORTS_PER_SOL;

            await program.methods
                .withdraw(new anchor.BN(overWithdraw))
                .accounts({
                    user: user.publicKey,
                    userAccount: pda,
                    systemProgram: SystemProgram.programId,
                })
                .signers([user])
                .rpc();

            assert.fail("Withdraw succeeded unexpectedly, should have failed.");
        } catch (err) {
            const errMsg = err.error?.errorMessage || err.toString();
            assert.include(errMsg, "Insufficient balance");
        }
    });
});