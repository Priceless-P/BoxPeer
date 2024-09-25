import { Aptos, AptosConfig, KeylessAccount, Network } from "@aptos-labs/ts-sdk"; // Import types from Aptos SDK

const aptosConfig = new AptosConfig({ network: Network.TESTNET });
const aptos = new Aptos(aptosConfig);

const moduleAddress = "0x9e99af6d494ca087085ae7b14c0f422b41b53e62db5b68708bbb2286f8abcb45";

export const upload_content = async (
  account: KeylessAccount,
  content_hash: string,
  nodes: string[],
  fee_paid: number,
  consumer_fee: number
) => {
  try {
    // Build a raw transaction
    const rawTransaction = await aptos.transaction.build.simple({
      sender: account.accountAddress,
      data: {
        function: `${moduleAddress}::BoxPeer::upload_content`,
        typeArguments: [],
        functionArguments: [content_hash, nodes, fee_paid, consumer_fee]
      }
    });

    const committedTxn = await aptos.signAndSubmitTransaction({ signer: account, transaction: rawTransaction });
    await aptos.waitForTransaction({ transactionHash: committedTxn.hash });
    console.log(`Committed transaction: ${committedTxn.hash}`);

  } catch (error: any) {
    console.error('Error submitting transaction:', error);
  }
};
