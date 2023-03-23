import { RpcClientModule } from "@dcl/rpc/dist/codegen"
import { QuestsServiceDefinition, QuestState, Action } from "./quests"

export type QuestStates = Record<string, QuestState>
export type StateUpdateCallback = (state: QuestStates) => void
export interface StartClient {
  start: (userAddress: string) => Promise<QuestsClient>
}
export type QuestsClient = RpcClientModule<QuestsServiceDefinition> & {
  // Listen to Quest State updates
  onQuestStateUpdate: (callback: StateUpdateCallback) => void
  // Check if an action would make progress in any quest
  checkAction: (action: Action) => boolean
}
