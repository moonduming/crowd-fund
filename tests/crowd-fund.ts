import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { CrowdFund } from "../target/types/crowd_fund";
import { 
  Keypair, 
  LAMPORTS_PER_SOL, 
  PublicKey, 
  SystemProgram, 
  Transaction 
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  createMint,
  mintTo,
  getOrCreateAssociatedTokenAccount,
} from "@solana/spl-token";

describe("crowd-fund", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const connection = provider.connection;

  const program = anchor.workspace.CrowdFund as Program<CrowdFund>;

  // 存储 payer (当前 provider.wallet)
  const payer = (provider.wallet as anchor.Wallet).payer;

  // SPL 代币 mint 地址
  let mint: PublicKey;

  // 捐赠者账户数组
  const donors: { keypair: Keypair; tokenAccount: PublicKey }[] = [];

  // 捐赠者数量
  const NUM_DONORS = 4;
  // 每个捐赠者需要空投的 SOL 数量
  const AIRDROP_AMOUNT = 2 * LAMPORTS_PER_SOL;

  // 模拟 SOL 空投 (本地环境没有requestAirdrop的替代方案)
  async function fundAccount(from: Keypair, to: PublicKey, amount: number) {
    const tx = new Transaction().add(
      SystemProgram.transfer({
        fromPubkey: from.publicKey,
        toPubkey: to,
        lamports: amount,
      })
    );
    const signature = await provider.sendAndConfirm(tx, [from]);
    return signature;
  }

  before(async () => {
    // 1. 创建一个新的 SPL 代币 mint (小数位为9，类似SOL)
    mint = await createMint(
      connection,
      payer,
      payer.publicKey,
      null,
      2,
    );
    console.log("Mint created:", mint.toBase58());

    // 2. 创建捐赠者账户，转入SOL，创建关联token账户并铸造代币
    for (let i = 0; i < NUM_DONORS; i++) {
      const donorKeypair = Keypair.generate();

      // 模拟 SOL 空投（直接转账）
      await fundAccount(payer, donorKeypair.publicKey, AIRDROP_AMOUNT);
      console.log(`Donor ${i + 1} funded with SOL:`, donorKeypair.publicKey.toBase58());

      // 创建关联代币账户 (ATA)
      const donorTokenAccount = await getOrCreateAssociatedTokenAccount(
        connection,
        payer,
        mint,
        donorKeypair.publicKey,
      );

      // 向捐赠者账户铸造一些代币 (例如1000枚)
      const mintAmount = 1000 * 100; // 考虑到小数位
      await mintTo(
        connection,
        payer,
        mint,
        donorTokenAccount.address,
        payer,
        mintAmount,
      );
      console.log(`Donor ${i + 1} minted tokens:`, donorTokenAccount.address.toBase58());

      donors.push({
        keypair: donorKeypair,
        tokenAccount: donorTokenAccount.address,
      });
    }
  });

  it("should initialize the campaign correctly", async () => {
    // 调用初始化 campaign 指令时需要带上完整的accounts
    await program.methods.campaign(
      "捐款测试",
      new anchor.BN(10000),
      new anchor.BN(1738915524),
      new anchor.BN(1744013124)
    ).accounts({
      mint,
      tokenProgram: TOKEN_PROGRAM_ID,
    }).rpc();
  
    // 获取并打印 campaign 数据，确认成功初始化
    const [crowdfundAccountPda] = PublicKey.findProgramAddressSync(
      [provider.wallet.publicKey.toBuffer()],
      program.programId
    );

    const campaignData = await program.account.crowdfund.fetch(crowdfundAccountPda);
    console.log("Campaign Data:", campaignData);
  });

  it("should handle multiple concurrent donations correctly", async () => {
    const DONATION_AMOUNT = new anchor.BN(45 * 100); // 每个捐赠者捐赠500个代币
  
    // 构建多个并行的捐款请求
    await Promise.all(
      donors.map(async ({ keypair, tokenAccount }, index) => {
        try {
          const donorProvider = new anchor.AnchorProvider(
            connection,
            new anchor.Wallet(keypair),
            { commitment: "confirmed" }
          );
          const donorProgram = new Program<CrowdFund>(
            program.idl as CrowdFund,
            donorProvider
          );
    
          // 每个donor单独计算自己的 donation_record_account PDA
          const [donationRecordAccountPda] = PublicKey.findProgramAddressSync(
            [keypair.publicKey.toBuffer()],
            program.programId
          );
          
          // 调用捐款指令
          const tx = await donorProgram.methods
            .donation(DONATION_AMOUNT)
            .accounts({
              donor: keypair.publicKey,
              maker: payer.publicKey,
              mint,
              tokenProgram: TOKEN_PROGRAM_ID,
            })
            .rpc({ commitment: "confirmed" });
    
          console.log(`Donor ${index + 1} donation tx: ${tx}`);
        } catch (error: any) {
          console.error(`Donation for donor ${index + 1} failed: ${error.message}`);
        };
        
      })
    );
  
    const [crowdfundAccountPda] = PublicKey.findProgramAddressSync(
      [provider.wallet.publicKey.toBuffer()],
      program.programId
    );

    const campaignData = await program.account.crowdfund.fetch(crowdfundAccountPda);
    console.log("Campaign Data:", campaignData);
    console.log("All donations completed concurrently.");
  });

  it("withdrawal", async () => {
    await program.methods.withdraw().accounts({
      mint,
      tokenProgram: TOKEN_PROGRAM_ID
    }).rpc()

    const [crowdfundAccountPda] = PublicKey.findProgramAddressSync(
      [provider.wallet.publicKey.toBuffer()],
      program.programId
    );

    const campaignData = await program.account.crowdfund.fetch(crowdfundAccountPda);
    console.log("Campaign Data:", campaignData);    
  })
});