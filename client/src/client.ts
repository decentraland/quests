import { RpcClientPort } from "@dcl/rpc"
import { loadService } from "@dcl/rpc/dist/codegen"
import { Action, QuestsServiceDefinition, QuestState } from "./quests"
import { createRpcClient, RpcClient } from "@dcl/rpc"
import { WebSocketTransport } from "@dcl/rpc/dist/transports/WebSocket"
import { WebSocket } from "ws"
import { StartClient, StateUpdateCallback } from "./types"

type ClientState = {
  questStates: Record<string, QuestState>
  processingEvents: Array<{ eventId: number; action: Action }>
  callbacks: Array<StateUpdateCallback>
}

/**
 * Creates an RPC Client that implements the QuestsService
 * ready to subscribe to a userAddress
 * @public
 */
export async function createQuestsClient(ws: string): Promise<StartClient> {
  const rpcClient = await createWebSocketRpcClient(ws)
  const port = await rpcClient.createPort("quests-client")
  const client = createQuestsServiceClient(port)

  // Keep state up to date on every received update
  async function subscribeAndUpdateState(state: ClientState, userAddress: string) {
    for await (const update of client.subscribe({ userAddress })) {
      // event sent had no impact on any quest state
      if (update.eventIgnored) {
        state.processingEvents = state.processingEvents.filter((event) => event.eventId === update.eventIgnored)
      }

      // there was an update on a quest state
      if (update.questState) {
        state.questStates[update.questState.questInstanceId] = update.questState
        for (const callback of state.callbacks) {
          callback(state.questStates)
        }
      }
    }
  }

  return {
    start: async (userAddress) => {
      const state = {
        questStates: {} as Record<string, QuestState>,
        processingEvents: new Array(),
        callbacks: new Array(),
      }

      // Subscribe to updates
      subscribeAndUpdateState(state, userAddress)

      return {
        ...client,
        checkAction: (action) => {
          // check if we are waiting for an event response with same action
          const processingEventWithSameAction = state.processingEvents.some((event) => event.action === action)

          // check if some quest expects this action
          const someQuestExpectsAction = () =>
            Object.values(state.questStates).some((questState) =>
              Object.values(questState.currentSteps).some((step) =>
                step.toDos.some((task) => task.actionItems.some((action_item) => action_item === action))
              )
            )

          return !processingEventWithSameAction && someQuestExpectsAction()
        },
        sendEvent: async (event) => {
          let eventResponse = await client.sendEvent(event)
          if (eventResponse.accepted && eventResponse.eventId) {
            state.processingEvents.push({ eventId: eventResponse.eventId, action: event.action })
          }
          return eventResponse
        },
        onQuestStateUpdate: (callback) => {
          state.callbacks.push(callback)
        },
      }
    },
  }
}

function createQuestsServiceClient<Context extends {}>(clientPort: RpcClientPort) {
  return loadService<Context, QuestsServiceDefinition>(clientPort, QuestsServiceDefinition)
}

async function createWebSocketRpcClient(wsUrl: string): Promise<RpcClient> {
  const ws = new WebSocket(wsUrl)
  const rpcClient = await createRpcClient(WebSocketTransport(ws as any))
  return rpcClient
}
