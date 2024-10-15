
import { Aptos, AptosConfig, KeylessAccount, U64 } from "@aptos-labs/ts-sdk";
import { Network } from "aptos";
const API_KEY = import.meta.env.VITE_API_KEY;
const moduleAddress = import.meta.env.VITE_moduleAddress;

const config = new AptosConfig({
  network: Network.TESTNET,
  fullnode: `https://aptos-testnet.nodit.io/${API_KEY}/v1`,
 indexer: `https://aptos-testnet.nodit.io/${API_KEY}/v1/graphql`,
});

const aptos = new Aptos(config);

export const upload_content = async (
    account: KeylessAccount,
    cid: string,
    fee_paid: U64,
    consumer_fee: U64,
    file_type: string,
    owner_name: string,
    description: string,
    title: string,
  ) => {
    try {
      // Build a simple transaction using aptos SDK
      const rawTransaction = await aptos.transaction.build.simple({
        sender: account.accountAddress.bcsToHex().toString(),
        data: {
            function: `${moduleAddress}::BoxPeer::upload_content`,
            typeArguments: [],
            functionArguments: [cid, fee_paid, consumer_fee, file_type, owner_name, description, title]
        }
    });

    const signedTxn = await aptos.signAndSubmitTransaction({ signer: account, transaction: rawTransaction });
      // Wait for the transaction to be confirmed
      await aptos.waitForTransaction({ transactionHash: signedTxn.hash });

      return `Success. Transaction hash ${signedTxn.hash}`;
    } catch (error: any) {
      console.error('Error submitting transaction:', error.message);
      return `Failed to upload content: ${error.message}`;
    }
  };

  export const payForContent = async (
    account: KeylessAccount,
    cid: string,
  ) => {
    try {
      const rawTransaction = await aptos.transaction.build.simple({
        sender: account.accountAddress.bcsToHex().toString(),
        data: {
            function: `${moduleAddress}::BoxPeer::pay_for_content`,
            typeArguments: [],
            functionArguments: [cid]
        }
    });

    const signedTxn = await aptos.signAndSubmitTransaction({ signer: account, transaction: rawTransaction });
      await aptos.waitForTransaction({ transactionHash: signedTxn.hash });
      return `Success. Transaction hash ${signedTxn.hash}`;
    } catch (error: any) {
      console.error('Error submitting transaction:', error.message);
      return `Failed to pay for content: ${error.message}`;
    }
  };

  export const getReward = async (
    account: KeylessAccount,
    cid: string,
    amount: U64
  ) => {
    try {
      const rawTransaction = await aptos.transaction.build.simple({
        sender: account.accountAddress.bcsToHex().toString(),
        data: {
            function: `${moduleAddress}::BoxPeer::get_reward`,
            typeArguments: [],
            functionArguments: [cid, amount]
        }
    });

    const signedTxn = await aptos.signAndSubmitTransaction({ signer: account, transaction: rawTransaction });
      await aptos.waitForTransaction({ transactionHash: signedTxn.hash });
      return `Success. Transaction hash ${signedTxn.hash}`;
    } catch (error: any) {
      console.error('Error submitting transaction:', error.message);
      return `Failed to pay for content: ${error.message}`;
    }
  };


// Function to query and fetch all Content events globally and collect CIDs
export const fetchAllContentCIDsNodit = async (): Promise<string[]> => {
    const queryContentEvents = `
query EventQuery {
  events(
    offset: 0
    limit: 100
    where: {
      account_address: {
        _eq: "0x61c8f3e7ecbcda5dd641c434b277a13b6076c09de32322ce197d2fe3f1e54ef1"
      }
      type: {_eq: "0x61c8f3e7ecbcda5dd641c434b277a13b6076c09de32322ce197d2fe3f1e54ef1::BoxPeer::Content"}
    }
  ) {
    type
    data
  }
}
  `;
    try {
        if (config.indexer) {
        const proxy = `http://localhost:3001/proxy/`
            const response = await fetch(`${proxy}`, {
                method: "POST",
                headers: {
                  "Content-Type": "application/json",
                },
                body: JSON.stringify({
                  query: queryContentEvents,
                }),
              });

              const result = await response.json();

              if (result.errors) {
                console.error("GraphQL query failed", result.errors);
                return [];
              }
              console.log(result.data.json)
      const events = result.data?.events || [];
      const cids = events.map((event: any) => event.data);

      return cids;
        }
        return [];

    } catch (error: any) {
      console.error("Error fetching content events:", error);
      return [];
    }
  };
  export const fetchAllContentCIDs = async (): Promise<string[]> => {
    const accountAddress = "0x61c8f3e7ecbcda5dd641c434b277a13b6076c09de32322ce197d2fe3f1e54ef1";
    const eventType = `${moduleAddress}::BoxPeer::Content` as `${string}::${string}::${string}`;
    try {
      // Fetch events using the SDK
        const events = await aptos.event.getAccountEventsByEventType({
            accountAddress,
            eventType: eventType,
      });

      // Map events to extract the CIDs or other relevant data
      const cids = events.map((event: any) => event.data);
    //   console.log("Events:", cids)
      return cids;
    } catch (error: any) {
      console.error("Error fetching content events:", error);
      return [];
    }
  };
export const getPurchasersByCid = async(cid: string) => {
    const result = await aptos.view({
        payload: {
            function: `${moduleAddress}::BoxPeer::get_purchasers_by_cid`,
            typeArguments: [],
            functionArguments: [cid],
        },
    });
    // console.log(result)
    return result;
}

export const getTotalEarned = async(owner: string) => {
    const result = await aptos.view({
        payload: {
            function: `${moduleAddress}::BoxPeer::get_total_earned_by_owner`,
            typeArguments: [],
            functionArguments: [owner],
        },
    });
    console.log(result)
    const totalEarnedMicroAPT = result[0] || 0;
    const totalEarnedAPT = Number(totalEarnedMicroAPT) / 1e8;

    return totalEarnedAPT.toFixed(3)
}
