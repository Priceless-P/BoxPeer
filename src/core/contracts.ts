import { Aptos, AptosConfig, KeylessAccount, Network, U64 } from "@aptos-labs/ts-sdk";

const aptosConfig = new AptosConfig({ network: Network.TESTNET });
const aptos = new Aptos(aptosConfig);

const moduleAddress = "001cdc16ec30101c48a9c0f069c7570f6ca50a66a67615676e90490219d77a08";

export const upload_content = async (
  account: KeylessAccount,
  content_hash: string,
  nodes: string[],
  fee_paid: U64,
  consumer_fee: U64
) => {
  try {
    // Build a raw transaction
    const rawTransaction = await aptos.transaction.build.simple({
      sender: account.accountAddress.bcsToHex().toString(),
      data: {
        function: `${moduleAddress}::BoxPeer::upload_content`,
        typeArguments: [],
        functionArguments: [content_hash, nodes, fee_paid, consumer_fee]
      }
    });

    const committedTxn = await aptos.signAndSubmitTransaction({ signer: account, transaction: rawTransaction });
    await aptos.waitForTransaction({ transactionHash: committedTxn.hash });
    return (`Success. Transaction hash ${committedTxn.hash}`);

  } catch (error: any) {
    console.error('Error submitting transaction:', error);
  }
};
