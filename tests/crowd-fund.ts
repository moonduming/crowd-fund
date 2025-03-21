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
import * as fs from "fs";
import { Client as PgClient } from "pg";
import { MerkleTree } from "merkletreejs";
import crypto from "crypto";
import { on } from "events";

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
  const NUM_DONORS = 6;
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
  };

  // 使用 SHA-256 作为哈希函数
  function sha256(data: Buffer): Buffer {
    return crypto.createHash("sha256").update(data).digest();
  };

  interface DonorInfo {
    keypair: Keypair;
    publicKey: PublicKey;
    donationAmount: number;
    tokenAccount: PublicKey;
  }


  // 从 donors.json 中加载捐赠者信息（假设文件中保存了 secretKey 和 publicKey 字符串）
  function loadDonors(): DonorInfo[] {
    // 读取 donors.json 文件（请根据实际路径调整）
    const donorsData = JSON.parse(fs.readFileSync("donors.json", "utf8"));

    // donorsData 中每个对象包含 secretKey, publicKey, tokenAccount（存储为字符串）
    return donorsData.map((donor: any, index: number) => {
      // 根据索引分配捐赠金额：偶数索引 4500，奇数索引 3000
      const donationAmount = (index % 2 === 0) ? 4500 : 3000;
      return {
        keypair: Keypair.fromSecretKey(new Uint8Array(donor.secretKey)),
        publicKey: new PublicKey(donor.publicKey),
        donationAmount,
        tokenAccount: new PublicKey(donor.tokenAccount)
      };
    });
  };

  function getMerkleProofForLeaf(tree: MerkleTree, leaf: Buffer): number[][] {
    // tree.getProof 返回一个数组，其中每个元素形如 { position: 'left' | 'right', data: Buffer }
    const proofBuffers: Buffer[] = tree.getProof(leaf).map(x => x.data);
    // 转换为数字数组数组
    return proofBuffers.map(buf => Array.from(buf));
  }

  function printMerkleTree(tree: MerkleTree): void {
    // 如果库版本支持 getLayers() 方法
    const layers = tree.getLayers ? tree.getLayers() : (tree as any)._layers;
    if (!layers) {
      console.error("无法获取树的层级数据");
      return;
    }
    layers.forEach((level: Buffer[], index: number) => {
      const levelHex = level.map(node => node.toString("hex"));
      console.log(`Layer ${index}:`, levelHex);
    });
  }

  // 计算 Root 并验证
function verifyMerkleProof(leaf: Buffer, proof: number[][], root: Buffer): Buffer {
    let computedHash = leaf;

    for (const proofElement of proof) {
      let sibling = Buffer.from(proofElement);
      let combined;

      // 确保排序顺序与构造时一致
      if (Buffer.compare(computedHash, sibling) < 0) {
          combined = Buffer.concat([computedHash, sibling]);
      } else {
          combined = Buffer.concat([sibling, computedHash]);
      }

      computedHash = sha256(combined);
    }

    return computedHash;
  }

  // 从 PostgreSQL 查询最新的 Merkle Tree 数据（假设表名为 merkle_trees，字段 tree_data 为 JSONB）
  async function getMerkleTreeData(): Promise<{ merkleRoot: string; leaves: Buffer[] }> {
    const pgClient = new PgClient({
      host: "localhost",
      database: "postgres",
      user: "postgres",
      password: "123456",
    });
    await pgClient.connect();
    const res = await pgClient.query("SELECT merkle_root, tree_data FROM merkle_trees ORDER BY id DESC LIMIT 1");
    await pgClient.end();
    if (res.rows.length === 0) {
      throw new Error("No Merkle Tree record found in PostgreSQL");
    }
    const merkleRootHex: string = res.rows[0].merkle_root;
    const treeData: string[][] = res.rows[0].tree_data; // 假设 tree_data 是二维数组，第一层为 leaves（十六进制字符串）
    // 将第一层叶子转换为 Buffer
    const leaves = treeData[0].map(hexStr => Buffer.from(hexStr, "hex"));
    
    return { merkleRoot: merkleRootHex, leaves };
  }
  // before(async () => {
  //   // 1. 创建一个新的 SPL 代币 mint (小数位为9，类似SOL)
  //   mint = await createMint(
  //     connection,
  //     payer,
  //     payer.publicKey,
  //     null,
  //     2,
  //   );
  //   console.log("Mint created:", mint.toBase58());

  //   // 2. 创建捐赠者账户，转入SOL，创建关联token账户并铸造代币
  //   for (let i = 0; i < NUM_DONORS; i++) {
  //     const donorKeypair = Keypair.generate();

  //     // 模拟 SOL 空投（直接转账）
  //     await fundAccount(payer, donorKeypair.publicKey, AIRDROP_AMOUNT);
  //     console.log(`Donor ${i + 1} funded with SOL:`, donorKeypair.publicKey.toBase58());

  //     // 创建关联代币账户 (ATA)
  //     const donorTokenAccount = await getOrCreateAssociatedTokenAccount(
  //       connection,
  //       payer,
  //       mint,
  //       donorKeypair.publicKey,
  //     );

  //     // 向捐赠者账户铸造一些代币 (例如1000枚)
  //     const mintAmount = 1000 * 100; // 考虑到小数位
  //     await mintTo(
  //       connection,
  //       payer,
  //       mint,
  //       donorTokenAccount.address,
  //       payer,
  //       mintAmount,
  //     );
  //     console.log(`Donor ${i + 1} minted tokens:`, donorTokenAccount.address.toBase58());

  //     donors.push({
  //       keypair: donorKeypair,
  //       tokenAccount: donorTokenAccount.address,
  //     });
  //   };

  //   // 保存 donors 到 donors.json 文件
  //   const donorsToSave = donors.map(({ keypair, tokenAccount }) => ({
  //     secretKey: Array.from(keypair.secretKey), // 保存为数组
  //     publicKey: keypair.publicKey.toBase58(),
  //     tokenAccount,
  //   }));
  //   fs.writeFileSync("donors.json", JSON.stringify(donorsToSave, null, 2), "utf-8");
  //   console.log("Donor data saved to donors.json");
  // });

  // it("should initialize the campaign correctly", async () => {
  //   const now = Math.floor(Date.now() / 1000);

  //   await program.methods.campaign(
  //     "捐款测试",
  //     new anchor.BN(20000),
  //     new anchor.BN(now - 3600),
  //     new anchor.BN(now + 60)
  //   ).accounts({
  //     mint,
  //     tokenProgram: TOKEN_PROGRAM_ID,
  //   }).rpc();

  //   // 获取并打印 campaign 数据，确认成功初始化
  //   const [crowdfundAccountPda] = PublicKey.findProgramAddressSync(
  //     [provider.wallet.publicKey.toBuffer()],
  //     program.programId
  //   );

  //   const campaignData = await program.account.crowdfund.fetch(crowdfundAccountPda);
  //   console.log("Campaign Data:", campaignData);
  // });

  // it("should handle multiple concurrent donations correctly", async () => {

  //   const DONATION_AMOUNT = new anchor.BN(45 * 100);

  //   // 构建多个并行的捐款请求
  //   await Promise.all(
  //     donors.map(async ({ keypair, tokenAccount }, index) => {
  //       try {
  //         const donorProvider = new anchor.AnchorProvider(
  //           connection,
  //           new anchor.Wallet(keypair),
  //           { commitment: "confirmed" }
  //         );
  //         const donorProgram = new Program<CrowdFund>(
  //           program.idl as CrowdFund,
  //           donorProvider
  //         );

  //         // const DONATION_AMOUNT = (index % 2 === 0)
  //         //   ? new anchor.BN(45 * 100)
  //         //   : new anchor.BN(30 * 100);

  //         // 调用捐款指令
  //         const tx = await donorProgram.methods
  //           .donation(DONATION_AMOUNT)
  //           .accounts({
  //             donor: keypair.publicKey,
  //             maker: payer.publicKey,
  //             mint,
  //             tokenProgram: TOKEN_PROGRAM_ID,
  //           })
  //           .rpc({ commitment: "confirmed" });

  //         console.log(`Donor ${index + 1} donation tx: ${tx}`);
  //       } catch (error: any) {
  //         console.error(`Donation for donor ${index + 1} failed: ${error.message}`);
  //       };

  //     })
  //   );

  //   const [crowdfundAccountPda] = PublicKey.findProgramAddressSync(
  //     [provider.wallet.publicKey.toBuffer()],
  //     program.programId
  //   );

  //   const campaignData = await program.account.crowdfund.fetch(crowdfundAccountPda);
  //   console.log("Campaign Data:", campaignData);
  //   console.log("All donations completed concurrently.");
  // });

  // it("withdrawal", async () => {
  //   try {
  //     await program.methods.withdraw().accounts({
  //       mint,
  //       tokenProgram: TOKEN_PROGRAM_ID
  //     }).rpc()

  //     const [crowdfundAccountPda] = PublicKey.findProgramAddressSync(
  //       [provider.wallet.publicKey.toBuffer()],
  //       program.programId
  //     );

  //     const campaignData = await program.account.crowdfund.fetch(crowdfundAccountPda);
  //     console.log("Campaign Data:", campaignData); 
  //   } catch (error: any) {
  //     console.error("withdrawal err: ", error)
  //   }

  // })

  // it("refund", async () => {
  //   await program.methods.finalize().accounts({
  //     make: payer.publicKey
  //   }).rpc();

  //   await Promise.all(
  //     donors.map(async ({ keypair, tokenAccount }, index) => {
  //       try {
  //         const donorProvider = new anchor.AnchorProvider(
  //           connection,
  //           new anchor.Wallet(keypair),
  //           { commitment: "confirmed" }
  //         );
  //         const donorProgram = new Program<CrowdFund>(
  //           program.idl as CrowdFund,
  //           donorProvider
  //         );

  //         // 每个donor单独计算自己的 donation_record_account PDA
  //         const [donationRecordAccountPda] = PublicKey.findProgramAddressSync(
  //           [keypair.publicKey.toBuffer()],
  //           program.programId
  //         );

  //         await donorProgram.methods
  //           .refund()
  //           .accounts({
  //             donor: keypair.publicKey,
  //             weeklyPlanner: payer.publicKey,
  //             mint,
  //             tokenProgram: TOKEN_PROGRAM_ID,
  //           })
  //           .rpc();

  //         const donorData = await program.account.donationRecord.fetch(donationRecordAccountPda);
  //         console.log("donorData: ", donorData);
  //       } catch (error: any) {
  //         console.error(`Donation for donor ${index + 1} failed: ${error.message}`);
  //       };

  //     })
  //   );
  // })

  // it("save merkle root", async () => {
  //   const merkleRootBuffer = Buffer.from(
  //       "7a142ecc0f12a86f764bd2849d216ae1e19c6aaa4a4dda516246a9148cf2eede",
  //       "hex"
  //     );
  //   const merkleRootArray = Array.from(merkleRootBuffer);
  //   await program.methods.setMerkleRoot(
  //       merkleRootArray
  //     ).accounts({
  //       authority: payer.publicKey
  //     }).rpc();

  //   const [crowdfundAccountPda] = PublicKey.findProgramAddressSync(
  //     [provider.wallet.publicKey.toBuffer()],
  //     program.programId
  //   );

  //   const campaignData = await program.account.crowdfund.fetch(crowdfundAccountPda);
  //   console.log("Campaign Data:", campaignData);
  // });

  it("should allow an eligible donor to claim reward", async () => {
    // 获取捐赠者列表
    const donors = loadDonors();

    // 1. 从数据库中读取离线构造好的 Merkle Tree 数据
    const { merkleRoot, leaves } = await getMerkleTreeData();

    // 使用 merkletreejs 重新构造 Merkle Tree，确保用同样的哈希函数和合并规则
    const tree = new MerkleTree(leaves, sha256, { sortPairs: true, duplicateOdd: true });
    // const computedRoot = tree.getRoot().toString("hex");
    const computedRootArray = Array.from(tree.getRoot());
    console.log("root: ", computedRootArray)
    // printMerkleTree(tree);

    // 2. 假设我们选择第一个符合条件的捐赠者（中奖者）
    const donorInfo = donors[0]; // 此处假设 donors 中的记录都是中奖者
    // 构造叶子节点：这里的构造方式必须与离线构造时完全一致
    // 例如离线构造时使用的方式是： leaf_input = "{donor}-{donationAmount}"
    const leafInput = `${donorInfo.publicKey.toBase58()}-${donorInfo.donationAmount}`;
    const donorLeaf = sha256(Buffer.from(leafInput, "utf8"));

    // 3. 生成该捐赠者的 Merkle Proof（proof 数组）
    const proofArray = getMerkleProofForLeaf(tree, donorLeaf);
    console.log("proof:", proofArray);

    const computedRootFromProof = verifyMerkleProof(donorLeaf, proofArray, tree.getRoot());

    console.log("Computed Root from Proof (hex):", computedRootFromProof.toString("hex"));

    try {
      const tx = await program.methods.rewardClaim(proofArray)
        .accounts({
          donor: donorInfo.publicKey,
          maker: payer.publicKey, // 这里假设 maker 为发起人或管理员
        }).signers([donorInfo.keypair])
        .rpc({ commitment: "confirmed" });
      console.log("Reward claim transaction:", tx);
    } catch (error: any) {
      console.error("Reward claim failed:", error);
      throw error;
    }
  });
});