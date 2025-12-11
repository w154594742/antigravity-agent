import {create} from "zustand";
import {AntigravityAccountData} from "@/commands/types/account.types.ts";
import {CloudCodeAPI} from "@/services/cloudcode-api.ts";
import {CloudCodeAPITypes} from "@/services/cloudcode-api.types.ts";

type State = {
  data: Record<string, CloudCodeAPITypes.FetchAvailableModelsResponse>
}

type Actions = {
  fetchData: (antigravityAccount: AntigravityAccountData) => Promise<void>
}

export const useAvailableModels = create<State & Actions>((setState, getState) => ({
  data: {},
  fetchData: async (antigravityAccount: AntigravityAccountData) => {
    const codeAssistResponse = await CloudCodeAPI.loadCodeAssist(antigravityAccount.auth.access_token);

    const modelsResponse = await CloudCodeAPI.fetchAvailableModels(antigravityAccount.auth.access_token, codeAssistResponse.cloudaicompanionProject);

    setState({
      data: {
        ...getState().data,
        [antigravityAccount.context.email]: modelsResponse
      }
    })
  }
}))
