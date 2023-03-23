/* eslint-disable */
import _m0 from "protobufjs/minimal"

export const protobufPackage = "decentraland.quests"

export interface User {
  userAddress: string
}

export interface StartQuestRequest {
  userAddress: string
  questId: string
}

export interface StartQuestResponse {
  /**
   * There are a few valid reasons not to be accepted:
   *  - Quest is not found
   *  - Quest is deactivated (the owner deleted it)
   *  - User already started the quest
   *  - Internal errors (DB connection failed or something like that)
   */
  accepted: boolean
}

export interface AbortQuestRequest {
  userAddress: string
  questInstanceId: string
}

export interface AbortQuestResponse {
  /**
   * There are a few valid reasons not to be accepted:
   *  - Quest instance is not found
   *  - Quest instance is from another user
   *  - Quest instance already aborted
   *  - Internal errors (DB connection failed or something like that)
   */
  accepted: boolean
}

export interface Event {
  userAddress: string
  action: Action | undefined
}

export interface EventResponse {
  eventId?: number | undefined
  accepted: boolean
}

/**
 * Example:
 * Action {
 *   type: "Location",
 *   parameters: {
 *     x: 10,
 *     y: 10,
 *   }
 * }
 */
export interface Action {
  type: string
  parameters: { [key: string]: string }
}

export interface Action_ParametersEntry {
  key: string
  value: string
}

export interface Task {
  id: string
  description?: string | undefined
  actionItems: Action[]
}

export interface StepContent {
  toDos: Task[]
  taskCompleted: string[]
}

export interface QuestState {
  questInstanceId: string
  /**
   * Every step has one or more tasks.
   * Tasks description and completed tasks are tracked here.
   */
  currentSteps: { [key: string]: StepContent }
  stepsLeft: number
  stepsCompleted: string[]
  requiredSteps: string[]
}

export interface QuestState_CurrentStepsEntry {
  key: string
  value: StepContent | undefined
}

export interface UserUpdate {
  questState?: QuestState | undefined
  eventIgnored?: number | undefined
}

function createBaseUser(): User {
  return { userAddress: "" }
}

export const User = {
  encode(message: User, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.userAddress !== "") {
      writer.uint32(10).string(message.userAddress)
    }
    if ("_unknownFields" in message) {
      const msgUnknownFields: any = (message as any)["_unknownFields"]
      for (const key of Object.keys(msgUnknownFields)) {
        const values = msgUnknownFields[key] as Uint8Array[]
        for (const value of values) {
          writer.uint32(parseInt(key, 10))
          ;(writer as any)["_push"](
            (val: Uint8Array, buf: Buffer, pos: number) => buf.set(val, pos),
            value.length,
            value
          )
        }
      }
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): User {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseUser()
    ;(message as any)._unknownFields = {}
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          if (tag != 10) {
            break
          }

          message.userAddress = reader.string()
          continue
      }
      if ((tag & 7) == 4 || tag == 0) {
        break
      }
      const startPos = reader.pos
      reader.skipType(tag & 7)
      const buf = reader.buf.slice(startPos, reader.pos)
      const list = (message as any)._unknownFields[tag]

      if (list === undefined) {
        ;(message as any)._unknownFields[tag] = [buf]
      } else {
        list.push(buf)
      }
    }
    return message
  },

  fromJSON(object: any): User {
    return { userAddress: isSet(object.userAddress) ? String(object.userAddress) : "" }
  },

  toJSON(message: User): unknown {
    const obj: any = {}
    message.userAddress !== undefined && (obj.userAddress = message.userAddress)
    return obj
  },

  create<I extends Exact<DeepPartial<User>, I>>(base?: I): User {
    return User.fromPartial(base ?? {})
  },

  fromPartial<I extends Exact<DeepPartial<User>, I>>(object: I): User {
    const message = createBaseUser()
    message.userAddress = object.userAddress ?? ""
    return message
  },
}

function createBaseStartQuestRequest(): StartQuestRequest {
  return { userAddress: "", questId: "" }
}

export const StartQuestRequest = {
  encode(message: StartQuestRequest, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.userAddress !== "") {
      writer.uint32(10).string(message.userAddress)
    }
    if (message.questId !== "") {
      writer.uint32(18).string(message.questId)
    }
    if ("_unknownFields" in message) {
      const msgUnknownFields: any = (message as any)["_unknownFields"]
      for (const key of Object.keys(msgUnknownFields)) {
        const values = msgUnknownFields[key] as Uint8Array[]
        for (const value of values) {
          writer.uint32(parseInt(key, 10))
          ;(writer as any)["_push"](
            (val: Uint8Array, buf: Buffer, pos: number) => buf.set(val, pos),
            value.length,
            value
          )
        }
      }
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): StartQuestRequest {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseStartQuestRequest()
    ;(message as any)._unknownFields = {}
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          if (tag != 10) {
            break
          }

          message.userAddress = reader.string()
          continue
        case 2:
          if (tag != 18) {
            break
          }

          message.questId = reader.string()
          continue
      }
      if ((tag & 7) == 4 || tag == 0) {
        break
      }
      const startPos = reader.pos
      reader.skipType(tag & 7)
      const buf = reader.buf.slice(startPos, reader.pos)
      const list = (message as any)._unknownFields[tag]

      if (list === undefined) {
        ;(message as any)._unknownFields[tag] = [buf]
      } else {
        list.push(buf)
      }
    }
    return message
  },

  fromJSON(object: any): StartQuestRequest {
    return {
      userAddress: isSet(object.userAddress) ? String(object.userAddress) : "",
      questId: isSet(object.questId) ? String(object.questId) : "",
    }
  },

  toJSON(message: StartQuestRequest): unknown {
    const obj: any = {}
    message.userAddress !== undefined && (obj.userAddress = message.userAddress)
    message.questId !== undefined && (obj.questId = message.questId)
    return obj
  },

  create<I extends Exact<DeepPartial<StartQuestRequest>, I>>(base?: I): StartQuestRequest {
    return StartQuestRequest.fromPartial(base ?? {})
  },

  fromPartial<I extends Exact<DeepPartial<StartQuestRequest>, I>>(object: I): StartQuestRequest {
    const message = createBaseStartQuestRequest()
    message.userAddress = object.userAddress ?? ""
    message.questId = object.questId ?? ""
    return message
  },
}

function createBaseStartQuestResponse(): StartQuestResponse {
  return { accepted: false }
}

export const StartQuestResponse = {
  encode(message: StartQuestResponse, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.accepted === true) {
      writer.uint32(8).bool(message.accepted)
    }
    if ("_unknownFields" in message) {
      const msgUnknownFields: any = (message as any)["_unknownFields"]
      for (const key of Object.keys(msgUnknownFields)) {
        const values = msgUnknownFields[key] as Uint8Array[]
        for (const value of values) {
          writer.uint32(parseInt(key, 10))
          ;(writer as any)["_push"](
            (val: Uint8Array, buf: Buffer, pos: number) => buf.set(val, pos),
            value.length,
            value
          )
        }
      }
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): StartQuestResponse {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseStartQuestResponse()
    ;(message as any)._unknownFields = {}
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          if (tag != 8) {
            break
          }

          message.accepted = reader.bool()
          continue
      }
      if ((tag & 7) == 4 || tag == 0) {
        break
      }
      const startPos = reader.pos
      reader.skipType(tag & 7)
      const buf = reader.buf.slice(startPos, reader.pos)
      const list = (message as any)._unknownFields[tag]

      if (list === undefined) {
        ;(message as any)._unknownFields[tag] = [buf]
      } else {
        list.push(buf)
      }
    }
    return message
  },

  fromJSON(object: any): StartQuestResponse {
    return { accepted: isSet(object.accepted) ? Boolean(object.accepted) : false }
  },

  toJSON(message: StartQuestResponse): unknown {
    const obj: any = {}
    message.accepted !== undefined && (obj.accepted = message.accepted)
    return obj
  },

  create<I extends Exact<DeepPartial<StartQuestResponse>, I>>(base?: I): StartQuestResponse {
    return StartQuestResponse.fromPartial(base ?? {})
  },

  fromPartial<I extends Exact<DeepPartial<StartQuestResponse>, I>>(object: I): StartQuestResponse {
    const message = createBaseStartQuestResponse()
    message.accepted = object.accepted ?? false
    return message
  },
}

function createBaseAbortQuestRequest(): AbortQuestRequest {
  return { userAddress: "", questInstanceId: "" }
}

export const AbortQuestRequest = {
  encode(message: AbortQuestRequest, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.userAddress !== "") {
      writer.uint32(10).string(message.userAddress)
    }
    if (message.questInstanceId !== "") {
      writer.uint32(18).string(message.questInstanceId)
    }
    if ("_unknownFields" in message) {
      const msgUnknownFields: any = (message as any)["_unknownFields"]
      for (const key of Object.keys(msgUnknownFields)) {
        const values = msgUnknownFields[key] as Uint8Array[]
        for (const value of values) {
          writer.uint32(parseInt(key, 10))
          ;(writer as any)["_push"](
            (val: Uint8Array, buf: Buffer, pos: number) => buf.set(val, pos),
            value.length,
            value
          )
        }
      }
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): AbortQuestRequest {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseAbortQuestRequest()
    ;(message as any)._unknownFields = {}
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          if (tag != 10) {
            break
          }

          message.userAddress = reader.string()
          continue
        case 2:
          if (tag != 18) {
            break
          }

          message.questInstanceId = reader.string()
          continue
      }
      if ((tag & 7) == 4 || tag == 0) {
        break
      }
      const startPos = reader.pos
      reader.skipType(tag & 7)
      const buf = reader.buf.slice(startPos, reader.pos)
      const list = (message as any)._unknownFields[tag]

      if (list === undefined) {
        ;(message as any)._unknownFields[tag] = [buf]
      } else {
        list.push(buf)
      }
    }
    return message
  },

  fromJSON(object: any): AbortQuestRequest {
    return {
      userAddress: isSet(object.userAddress) ? String(object.userAddress) : "",
      questInstanceId: isSet(object.questInstanceId) ? String(object.questInstanceId) : "",
    }
  },

  toJSON(message: AbortQuestRequest): unknown {
    const obj: any = {}
    message.userAddress !== undefined && (obj.userAddress = message.userAddress)
    message.questInstanceId !== undefined && (obj.questInstanceId = message.questInstanceId)
    return obj
  },

  create<I extends Exact<DeepPartial<AbortQuestRequest>, I>>(base?: I): AbortQuestRequest {
    return AbortQuestRequest.fromPartial(base ?? {})
  },

  fromPartial<I extends Exact<DeepPartial<AbortQuestRequest>, I>>(object: I): AbortQuestRequest {
    const message = createBaseAbortQuestRequest()
    message.userAddress = object.userAddress ?? ""
    message.questInstanceId = object.questInstanceId ?? ""
    return message
  },
}

function createBaseAbortQuestResponse(): AbortQuestResponse {
  return { accepted: false }
}

export const AbortQuestResponse = {
  encode(message: AbortQuestResponse, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.accepted === true) {
      writer.uint32(8).bool(message.accepted)
    }
    if ("_unknownFields" in message) {
      const msgUnknownFields: any = (message as any)["_unknownFields"]
      for (const key of Object.keys(msgUnknownFields)) {
        const values = msgUnknownFields[key] as Uint8Array[]
        for (const value of values) {
          writer.uint32(parseInt(key, 10))
          ;(writer as any)["_push"](
            (val: Uint8Array, buf: Buffer, pos: number) => buf.set(val, pos),
            value.length,
            value
          )
        }
      }
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): AbortQuestResponse {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseAbortQuestResponse()
    ;(message as any)._unknownFields = {}
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          if (tag != 8) {
            break
          }

          message.accepted = reader.bool()
          continue
      }
      if ((tag & 7) == 4 || tag == 0) {
        break
      }
      const startPos = reader.pos
      reader.skipType(tag & 7)
      const buf = reader.buf.slice(startPos, reader.pos)
      const list = (message as any)._unknownFields[tag]

      if (list === undefined) {
        ;(message as any)._unknownFields[tag] = [buf]
      } else {
        list.push(buf)
      }
    }
    return message
  },

  fromJSON(object: any): AbortQuestResponse {
    return { accepted: isSet(object.accepted) ? Boolean(object.accepted) : false }
  },

  toJSON(message: AbortQuestResponse): unknown {
    const obj: any = {}
    message.accepted !== undefined && (obj.accepted = message.accepted)
    return obj
  },

  create<I extends Exact<DeepPartial<AbortQuestResponse>, I>>(base?: I): AbortQuestResponse {
    return AbortQuestResponse.fromPartial(base ?? {})
  },

  fromPartial<I extends Exact<DeepPartial<AbortQuestResponse>, I>>(object: I): AbortQuestResponse {
    const message = createBaseAbortQuestResponse()
    message.accepted = object.accepted ?? false
    return message
  },
}

function createBaseEvent(): Event {
  return { userAddress: "", action: undefined }
}

export const Event = {
  encode(message: Event, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.userAddress !== "") {
      writer.uint32(10).string(message.userAddress)
    }
    if (message.action !== undefined) {
      Action.encode(message.action, writer.uint32(18).fork()).ldelim()
    }
    if ("_unknownFields" in message) {
      const msgUnknownFields: any = (message as any)["_unknownFields"]
      for (const key of Object.keys(msgUnknownFields)) {
        const values = msgUnknownFields[key] as Uint8Array[]
        for (const value of values) {
          writer.uint32(parseInt(key, 10))
          ;(writer as any)["_push"](
            (val: Uint8Array, buf: Buffer, pos: number) => buf.set(val, pos),
            value.length,
            value
          )
        }
      }
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Event {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseEvent()
    ;(message as any)._unknownFields = {}
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          if (tag != 10) {
            break
          }

          message.userAddress = reader.string()
          continue
        case 2:
          if (tag != 18) {
            break
          }

          message.action = Action.decode(reader, reader.uint32())
          continue
      }
      if ((tag & 7) == 4 || tag == 0) {
        break
      }
      const startPos = reader.pos
      reader.skipType(tag & 7)
      const buf = reader.buf.slice(startPos, reader.pos)
      const list = (message as any)._unknownFields[tag]

      if (list === undefined) {
        ;(message as any)._unknownFields[tag] = [buf]
      } else {
        list.push(buf)
      }
    }
    return message
  },

  fromJSON(object: any): Event {
    return {
      userAddress: isSet(object.userAddress) ? String(object.userAddress) : "",
      action: isSet(object.action) ? Action.fromJSON(object.action) : undefined,
    }
  },

  toJSON(message: Event): unknown {
    const obj: any = {}
    message.userAddress !== undefined && (obj.userAddress = message.userAddress)
    message.action !== undefined && (obj.action = message.action ? Action.toJSON(message.action) : undefined)
    return obj
  },

  create<I extends Exact<DeepPartial<Event>, I>>(base?: I): Event {
    return Event.fromPartial(base ?? {})
  },

  fromPartial<I extends Exact<DeepPartial<Event>, I>>(object: I): Event {
    const message = createBaseEvent()
    message.userAddress = object.userAddress ?? ""
    message.action =
      object.action !== undefined && object.action !== null ? Action.fromPartial(object.action) : undefined
    return message
  },
}

function createBaseEventResponse(): EventResponse {
  return { eventId: undefined, accepted: false }
}

export const EventResponse = {
  encode(message: EventResponse, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.eventId !== undefined) {
      writer.uint32(13).fixed32(message.eventId)
    }
    if (message.accepted === true) {
      writer.uint32(16).bool(message.accepted)
    }
    if ("_unknownFields" in message) {
      const msgUnknownFields: any = (message as any)["_unknownFields"]
      for (const key of Object.keys(msgUnknownFields)) {
        const values = msgUnknownFields[key] as Uint8Array[]
        for (const value of values) {
          writer.uint32(parseInt(key, 10))
          ;(writer as any)["_push"](
            (val: Uint8Array, buf: Buffer, pos: number) => buf.set(val, pos),
            value.length,
            value
          )
        }
      }
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): EventResponse {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseEventResponse()
    ;(message as any)._unknownFields = {}
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          if (tag != 13) {
            break
          }

          message.eventId = reader.fixed32()
          continue
        case 2:
          if (tag != 16) {
            break
          }

          message.accepted = reader.bool()
          continue
      }
      if ((tag & 7) == 4 || tag == 0) {
        break
      }
      const startPos = reader.pos
      reader.skipType(tag & 7)
      const buf = reader.buf.slice(startPos, reader.pos)
      const list = (message as any)._unknownFields[tag]

      if (list === undefined) {
        ;(message as any)._unknownFields[tag] = [buf]
      } else {
        list.push(buf)
      }
    }
    return message
  },

  fromJSON(object: any): EventResponse {
    return {
      eventId: isSet(object.eventId) ? Number(object.eventId) : undefined,
      accepted: isSet(object.accepted) ? Boolean(object.accepted) : false,
    }
  },

  toJSON(message: EventResponse): unknown {
    const obj: any = {}
    message.eventId !== undefined && (obj.eventId = Math.round(message.eventId))
    message.accepted !== undefined && (obj.accepted = message.accepted)
    return obj
  },

  create<I extends Exact<DeepPartial<EventResponse>, I>>(base?: I): EventResponse {
    return EventResponse.fromPartial(base ?? {})
  },

  fromPartial<I extends Exact<DeepPartial<EventResponse>, I>>(object: I): EventResponse {
    const message = createBaseEventResponse()
    message.eventId = object.eventId ?? undefined
    message.accepted = object.accepted ?? false
    return message
  },
}

function createBaseAction(): Action {
  return { type: "", parameters: {} }
}

export const Action = {
  encode(message: Action, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.type !== "") {
      writer.uint32(10).string(message.type)
    }
    Object.entries(message.parameters).forEach(([key, value]) => {
      Action_ParametersEntry.encode({ key: key as any, value }, writer.uint32(18).fork()).ldelim()
    })
    if ("_unknownFields" in message) {
      const msgUnknownFields: any = (message as any)["_unknownFields"]
      for (const key of Object.keys(msgUnknownFields)) {
        const values = msgUnknownFields[key] as Uint8Array[]
        for (const value of values) {
          writer.uint32(parseInt(key, 10))
          ;(writer as any)["_push"](
            (val: Uint8Array, buf: Buffer, pos: number) => buf.set(val, pos),
            value.length,
            value
          )
        }
      }
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Action {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseAction()
    ;(message as any)._unknownFields = {}
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          if (tag != 10) {
            break
          }

          message.type = reader.string()
          continue
        case 2:
          if (tag != 18) {
            break
          }

          const entry2 = Action_ParametersEntry.decode(reader, reader.uint32())
          if (entry2.value !== undefined) {
            message.parameters[entry2.key] = entry2.value
          }
          continue
      }
      if ((tag & 7) == 4 || tag == 0) {
        break
      }
      const startPos = reader.pos
      reader.skipType(tag & 7)
      const buf = reader.buf.slice(startPos, reader.pos)
      const list = (message as any)._unknownFields[tag]

      if (list === undefined) {
        ;(message as any)._unknownFields[tag] = [buf]
      } else {
        list.push(buf)
      }
    }
    return message
  },

  fromJSON(object: any): Action {
    return {
      type: isSet(object.type) ? String(object.type) : "",
      parameters: isObject(object.parameters)
        ? Object.entries(object.parameters).reduce<{ [key: string]: string }>((acc, [key, value]) => {
            acc[key] = String(value)
            return acc
          }, {})
        : {},
    }
  },

  toJSON(message: Action): unknown {
    const obj: any = {}
    message.type !== undefined && (obj.type = message.type)
    obj.parameters = {}
    if (message.parameters) {
      Object.entries(message.parameters).forEach(([k, v]) => {
        obj.parameters[k] = v
      })
    }
    return obj
  },

  create<I extends Exact<DeepPartial<Action>, I>>(base?: I): Action {
    return Action.fromPartial(base ?? {})
  },

  fromPartial<I extends Exact<DeepPartial<Action>, I>>(object: I): Action {
    const message = createBaseAction()
    message.type = object.type ?? ""
    message.parameters = Object.entries(object.parameters ?? {}).reduce<{ [key: string]: string }>(
      (acc, [key, value]) => {
        if (value !== undefined) {
          acc[key] = String(value)
        }
        return acc
      },
      {}
    )
    return message
  },
}

function createBaseAction_ParametersEntry(): Action_ParametersEntry {
  return { key: "", value: "" }
}

export const Action_ParametersEntry = {
  encode(message: Action_ParametersEntry, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.key !== "") {
      writer.uint32(10).string(message.key)
    }
    if (message.value !== "") {
      writer.uint32(18).string(message.value)
    }
    if ("_unknownFields" in message) {
      const msgUnknownFields: any = (message as any)["_unknownFields"]
      for (const key of Object.keys(msgUnknownFields)) {
        const values = msgUnknownFields[key] as Uint8Array[]
        for (const value of values) {
          writer.uint32(parseInt(key, 10))
          ;(writer as any)["_push"](
            (val: Uint8Array, buf: Buffer, pos: number) => buf.set(val, pos),
            value.length,
            value
          )
        }
      }
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Action_ParametersEntry {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseAction_ParametersEntry()
    ;(message as any)._unknownFields = {}
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          if (tag != 10) {
            break
          }

          message.key = reader.string()
          continue
        case 2:
          if (tag != 18) {
            break
          }

          message.value = reader.string()
          continue
      }
      if ((tag & 7) == 4 || tag == 0) {
        break
      }
      const startPos = reader.pos
      reader.skipType(tag & 7)
      const buf = reader.buf.slice(startPos, reader.pos)
      const list = (message as any)._unknownFields[tag]

      if (list === undefined) {
        ;(message as any)._unknownFields[tag] = [buf]
      } else {
        list.push(buf)
      }
    }
    return message
  },

  fromJSON(object: any): Action_ParametersEntry {
    return { key: isSet(object.key) ? String(object.key) : "", value: isSet(object.value) ? String(object.value) : "" }
  },

  toJSON(message: Action_ParametersEntry): unknown {
    const obj: any = {}
    message.key !== undefined && (obj.key = message.key)
    message.value !== undefined && (obj.value = message.value)
    return obj
  },

  create<I extends Exact<DeepPartial<Action_ParametersEntry>, I>>(base?: I): Action_ParametersEntry {
    return Action_ParametersEntry.fromPartial(base ?? {})
  },

  fromPartial<I extends Exact<DeepPartial<Action_ParametersEntry>, I>>(object: I): Action_ParametersEntry {
    const message = createBaseAction_ParametersEntry()
    message.key = object.key ?? ""
    message.value = object.value ?? ""
    return message
  },
}

function createBaseTask(): Task {
  return { id: "", description: undefined, actionItems: [] }
}

export const Task = {
  encode(message: Task, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.id !== "") {
      writer.uint32(10).string(message.id)
    }
    if (message.description !== undefined) {
      writer.uint32(18).string(message.description)
    }
    for (const v of message.actionItems) {
      Action.encode(v!, writer.uint32(26).fork()).ldelim()
    }
    if ("_unknownFields" in message) {
      const msgUnknownFields: any = (message as any)["_unknownFields"]
      for (const key of Object.keys(msgUnknownFields)) {
        const values = msgUnknownFields[key] as Uint8Array[]
        for (const value of values) {
          writer.uint32(parseInt(key, 10))
          ;(writer as any)["_push"](
            (val: Uint8Array, buf: Buffer, pos: number) => buf.set(val, pos),
            value.length,
            value
          )
        }
      }
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): Task {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseTask()
    ;(message as any)._unknownFields = {}
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          if (tag != 10) {
            break
          }

          message.id = reader.string()
          continue
        case 2:
          if (tag != 18) {
            break
          }

          message.description = reader.string()
          continue
        case 3:
          if (tag != 26) {
            break
          }

          message.actionItems.push(Action.decode(reader, reader.uint32()))
          continue
      }
      if ((tag & 7) == 4 || tag == 0) {
        break
      }
      const startPos = reader.pos
      reader.skipType(tag & 7)
      const buf = reader.buf.slice(startPos, reader.pos)
      const list = (message as any)._unknownFields[tag]

      if (list === undefined) {
        ;(message as any)._unknownFields[tag] = [buf]
      } else {
        list.push(buf)
      }
    }
    return message
  },

  fromJSON(object: any): Task {
    return {
      id: isSet(object.id) ? String(object.id) : "",
      description: isSet(object.description) ? String(object.description) : undefined,
      actionItems: Array.isArray(object?.actionItems) ? object.actionItems.map((e: any) => Action.fromJSON(e)) : [],
    }
  },

  toJSON(message: Task): unknown {
    const obj: any = {}
    message.id !== undefined && (obj.id = message.id)
    message.description !== undefined && (obj.description = message.description)
    if (message.actionItems) {
      obj.actionItems = message.actionItems.map((e) => (e ? Action.toJSON(e) : undefined))
    } else {
      obj.actionItems = []
    }
    return obj
  },

  create<I extends Exact<DeepPartial<Task>, I>>(base?: I): Task {
    return Task.fromPartial(base ?? {})
  },

  fromPartial<I extends Exact<DeepPartial<Task>, I>>(object: I): Task {
    const message = createBaseTask()
    message.id = object.id ?? ""
    message.description = object.description ?? undefined
    message.actionItems = object.actionItems?.map((e) => Action.fromPartial(e)) || []
    return message
  },
}

function createBaseStepContent(): StepContent {
  return { toDos: [], taskCompleted: [] }
}

export const StepContent = {
  encode(message: StepContent, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    for (const v of message.toDos) {
      Task.encode(v!, writer.uint32(10).fork()).ldelim()
    }
    for (const v of message.taskCompleted) {
      writer.uint32(18).string(v!)
    }
    if ("_unknownFields" in message) {
      const msgUnknownFields: any = (message as any)["_unknownFields"]
      for (const key of Object.keys(msgUnknownFields)) {
        const values = msgUnknownFields[key] as Uint8Array[]
        for (const value of values) {
          writer.uint32(parseInt(key, 10))
          ;(writer as any)["_push"](
            (val: Uint8Array, buf: Buffer, pos: number) => buf.set(val, pos),
            value.length,
            value
          )
        }
      }
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): StepContent {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseStepContent()
    ;(message as any)._unknownFields = {}
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          if (tag != 10) {
            break
          }

          message.toDos.push(Task.decode(reader, reader.uint32()))
          continue
        case 2:
          if (tag != 18) {
            break
          }

          message.taskCompleted.push(reader.string())
          continue
      }
      if ((tag & 7) == 4 || tag == 0) {
        break
      }
      const startPos = reader.pos
      reader.skipType(tag & 7)
      const buf = reader.buf.slice(startPos, reader.pos)
      const list = (message as any)._unknownFields[tag]

      if (list === undefined) {
        ;(message as any)._unknownFields[tag] = [buf]
      } else {
        list.push(buf)
      }
    }
    return message
  },

  fromJSON(object: any): StepContent {
    return {
      toDos: Array.isArray(object?.toDos) ? object.toDos.map((e: any) => Task.fromJSON(e)) : [],
      taskCompleted: Array.isArray(object?.taskCompleted) ? object.taskCompleted.map((e: any) => String(e)) : [],
    }
  },

  toJSON(message: StepContent): unknown {
    const obj: any = {}
    if (message.toDos) {
      obj.toDos = message.toDos.map((e) => (e ? Task.toJSON(e) : undefined))
    } else {
      obj.toDos = []
    }
    if (message.taskCompleted) {
      obj.taskCompleted = message.taskCompleted.map((e) => e)
    } else {
      obj.taskCompleted = []
    }
    return obj
  },

  create<I extends Exact<DeepPartial<StepContent>, I>>(base?: I): StepContent {
    return StepContent.fromPartial(base ?? {})
  },

  fromPartial<I extends Exact<DeepPartial<StepContent>, I>>(object: I): StepContent {
    const message = createBaseStepContent()
    message.toDos = object.toDos?.map((e) => Task.fromPartial(e)) || []
    message.taskCompleted = object.taskCompleted?.map((e) => e) || []
    return message
  },
}

function createBaseQuestState(): QuestState {
  return { questInstanceId: "", currentSteps: {}, stepsLeft: 0, stepsCompleted: [], requiredSteps: [] }
}

export const QuestState = {
  encode(message: QuestState, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.questInstanceId !== "") {
      writer.uint32(10).string(message.questInstanceId)
    }
    Object.entries(message.currentSteps).forEach(([key, value]) => {
      QuestState_CurrentStepsEntry.encode({ key: key as any, value }, writer.uint32(18).fork()).ldelim()
    })
    if (message.stepsLeft !== 0) {
      writer.uint32(29).fixed32(message.stepsLeft)
    }
    for (const v of message.stepsCompleted) {
      writer.uint32(34).string(v!)
    }
    for (const v of message.requiredSteps) {
      writer.uint32(42).string(v!)
    }
    if ("_unknownFields" in message) {
      const msgUnknownFields: any = (message as any)["_unknownFields"]
      for (const key of Object.keys(msgUnknownFields)) {
        const values = msgUnknownFields[key] as Uint8Array[]
        for (const value of values) {
          writer.uint32(parseInt(key, 10))
          ;(writer as any)["_push"](
            (val: Uint8Array, buf: Buffer, pos: number) => buf.set(val, pos),
            value.length,
            value
          )
        }
      }
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): QuestState {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseQuestState()
    ;(message as any)._unknownFields = {}
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          if (tag != 10) {
            break
          }

          message.questInstanceId = reader.string()
          continue
        case 2:
          if (tag != 18) {
            break
          }

          const entry2 = QuestState_CurrentStepsEntry.decode(reader, reader.uint32())
          if (entry2.value !== undefined) {
            message.currentSteps[entry2.key] = entry2.value
          }
          continue
        case 3:
          if (tag != 29) {
            break
          }

          message.stepsLeft = reader.fixed32()
          continue
        case 4:
          if (tag != 34) {
            break
          }

          message.stepsCompleted.push(reader.string())
          continue
        case 5:
          if (tag != 42) {
            break
          }

          message.requiredSteps.push(reader.string())
          continue
      }
      if ((tag & 7) == 4 || tag == 0) {
        break
      }
      const startPos = reader.pos
      reader.skipType(tag & 7)
      const buf = reader.buf.slice(startPos, reader.pos)
      const list = (message as any)._unknownFields[tag]

      if (list === undefined) {
        ;(message as any)._unknownFields[tag] = [buf]
      } else {
        list.push(buf)
      }
    }
    return message
  },

  fromJSON(object: any): QuestState {
    return {
      questInstanceId: isSet(object.questInstanceId) ? String(object.questInstanceId) : "",
      currentSteps: isObject(object.currentSteps)
        ? Object.entries(object.currentSteps).reduce<{ [key: string]: StepContent }>((acc, [key, value]) => {
            acc[key] = StepContent.fromJSON(value)
            return acc
          }, {})
        : {},
      stepsLeft: isSet(object.stepsLeft) ? Number(object.stepsLeft) : 0,
      stepsCompleted: Array.isArray(object?.stepsCompleted) ? object.stepsCompleted.map((e: any) => String(e)) : [],
      requiredSteps: Array.isArray(object?.requiredSteps) ? object.requiredSteps.map((e: any) => String(e)) : [],
    }
  },

  toJSON(message: QuestState): unknown {
    const obj: any = {}
    message.questInstanceId !== undefined && (obj.questInstanceId = message.questInstanceId)
    obj.currentSteps = {}
    if (message.currentSteps) {
      Object.entries(message.currentSteps).forEach(([k, v]) => {
        obj.currentSteps[k] = StepContent.toJSON(v)
      })
    }
    message.stepsLeft !== undefined && (obj.stepsLeft = Math.round(message.stepsLeft))
    if (message.stepsCompleted) {
      obj.stepsCompleted = message.stepsCompleted.map((e) => e)
    } else {
      obj.stepsCompleted = []
    }
    if (message.requiredSteps) {
      obj.requiredSteps = message.requiredSteps.map((e) => e)
    } else {
      obj.requiredSteps = []
    }
    return obj
  },

  create<I extends Exact<DeepPartial<QuestState>, I>>(base?: I): QuestState {
    return QuestState.fromPartial(base ?? {})
  },

  fromPartial<I extends Exact<DeepPartial<QuestState>, I>>(object: I): QuestState {
    const message = createBaseQuestState()
    message.questInstanceId = object.questInstanceId ?? ""
    message.currentSteps = Object.entries(object.currentSteps ?? {}).reduce<{ [key: string]: StepContent }>(
      (acc, [key, value]) => {
        if (value !== undefined) {
          acc[key] = StepContent.fromPartial(value)
        }
        return acc
      },
      {}
    )
    message.stepsLeft = object.stepsLeft ?? 0
    message.stepsCompleted = object.stepsCompleted?.map((e) => e) || []
    message.requiredSteps = object.requiredSteps?.map((e) => e) || []
    return message
  },
}

function createBaseQuestState_CurrentStepsEntry(): QuestState_CurrentStepsEntry {
  return { key: "", value: undefined }
}

export const QuestState_CurrentStepsEntry = {
  encode(message: QuestState_CurrentStepsEntry, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.key !== "") {
      writer.uint32(10).string(message.key)
    }
    if (message.value !== undefined) {
      StepContent.encode(message.value, writer.uint32(18).fork()).ldelim()
    }
    if ("_unknownFields" in message) {
      const msgUnknownFields: any = (message as any)["_unknownFields"]
      for (const key of Object.keys(msgUnknownFields)) {
        const values = msgUnknownFields[key] as Uint8Array[]
        for (const value of values) {
          writer.uint32(parseInt(key, 10))
          ;(writer as any)["_push"](
            (val: Uint8Array, buf: Buffer, pos: number) => buf.set(val, pos),
            value.length,
            value
          )
        }
      }
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): QuestState_CurrentStepsEntry {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseQuestState_CurrentStepsEntry()
    ;(message as any)._unknownFields = {}
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          if (tag != 10) {
            break
          }

          message.key = reader.string()
          continue
        case 2:
          if (tag != 18) {
            break
          }

          message.value = StepContent.decode(reader, reader.uint32())
          continue
      }
      if ((tag & 7) == 4 || tag == 0) {
        break
      }
      const startPos = reader.pos
      reader.skipType(tag & 7)
      const buf = reader.buf.slice(startPos, reader.pos)
      const list = (message as any)._unknownFields[tag]

      if (list === undefined) {
        ;(message as any)._unknownFields[tag] = [buf]
      } else {
        list.push(buf)
      }
    }
    return message
  },

  fromJSON(object: any): QuestState_CurrentStepsEntry {
    return {
      key: isSet(object.key) ? String(object.key) : "",
      value: isSet(object.value) ? StepContent.fromJSON(object.value) : undefined,
    }
  },

  toJSON(message: QuestState_CurrentStepsEntry): unknown {
    const obj: any = {}
    message.key !== undefined && (obj.key = message.key)
    message.value !== undefined && (obj.value = message.value ? StepContent.toJSON(message.value) : undefined)
    return obj
  },

  create<I extends Exact<DeepPartial<QuestState_CurrentStepsEntry>, I>>(base?: I): QuestState_CurrentStepsEntry {
    return QuestState_CurrentStepsEntry.fromPartial(base ?? {})
  },

  fromPartial<I extends Exact<DeepPartial<QuestState_CurrentStepsEntry>, I>>(object: I): QuestState_CurrentStepsEntry {
    const message = createBaseQuestState_CurrentStepsEntry()
    message.key = object.key ?? ""
    message.value =
      object.value !== undefined && object.value !== null ? StepContent.fromPartial(object.value) : undefined
    return message
  },
}

function createBaseUserUpdate(): UserUpdate {
  return { questState: undefined, eventIgnored: undefined }
}

export const UserUpdate = {
  encode(message: UserUpdate, writer: _m0.Writer = _m0.Writer.create()): _m0.Writer {
    if (message.questState !== undefined) {
      QuestState.encode(message.questState, writer.uint32(10).fork()).ldelim()
    }
    if (message.eventIgnored !== undefined) {
      writer.uint32(21).fixed32(message.eventIgnored)
    }
    if ("_unknownFields" in message) {
      const msgUnknownFields: any = (message as any)["_unknownFields"]
      for (const key of Object.keys(msgUnknownFields)) {
        const values = msgUnknownFields[key] as Uint8Array[]
        for (const value of values) {
          writer.uint32(parseInt(key, 10))
          ;(writer as any)["_push"](
            (val: Uint8Array, buf: Buffer, pos: number) => buf.set(val, pos),
            value.length,
            value
          )
        }
      }
    }
    return writer
  },

  decode(input: _m0.Reader | Uint8Array, length?: number): UserUpdate {
    const reader = input instanceof _m0.Reader ? input : _m0.Reader.create(input)
    let end = length === undefined ? reader.len : reader.pos + length
    const message = createBaseUserUpdate()
    ;(message as any)._unknownFields = {}
    while (reader.pos < end) {
      const tag = reader.uint32()
      switch (tag >>> 3) {
        case 1:
          if (tag != 10) {
            break
          }

          message.questState = QuestState.decode(reader, reader.uint32())
          continue
        case 2:
          if (tag != 21) {
            break
          }

          message.eventIgnored = reader.fixed32()
          continue
      }
      if ((tag & 7) == 4 || tag == 0) {
        break
      }
      const startPos = reader.pos
      reader.skipType(tag & 7)
      const buf = reader.buf.slice(startPos, reader.pos)
      const list = (message as any)._unknownFields[tag]

      if (list === undefined) {
        ;(message as any)._unknownFields[tag] = [buf]
      } else {
        list.push(buf)
      }
    }
    return message
  },

  fromJSON(object: any): UserUpdate {
    return {
      questState: isSet(object.questState) ? QuestState.fromJSON(object.questState) : undefined,
      eventIgnored: isSet(object.eventIgnored) ? Number(object.eventIgnored) : undefined,
    }
  },

  toJSON(message: UserUpdate): unknown {
    const obj: any = {}
    message.questState !== undefined &&
      (obj.questState = message.questState ? QuestState.toJSON(message.questState) : undefined)
    message.eventIgnored !== undefined && (obj.eventIgnored = Math.round(message.eventIgnored))
    return obj
  },

  create<I extends Exact<DeepPartial<UserUpdate>, I>>(base?: I): UserUpdate {
    return UserUpdate.fromPartial(base ?? {})
  },

  fromPartial<I extends Exact<DeepPartial<UserUpdate>, I>>(object: I): UserUpdate {
    const message = createBaseUserUpdate()
    message.questState =
      object.questState !== undefined && object.questState !== null
        ? QuestState.fromPartial(object.questState)
        : undefined
    message.eventIgnored = object.eventIgnored ?? undefined
    return message
  },
}

export type QuestsServiceDefinition = typeof QuestsServiceDefinition
export const QuestsServiceDefinition = {
  name: "QuestsService",
  fullName: "decentraland.quests.QuestsService",
  methods: {
    /** User actions */
    startQuest: {
      name: "StartQuest",
      requestType: StartQuestRequest,
      requestStream: false,
      responseType: StartQuestResponse,
      responseStream: false,
      options: {},
    },
    abortQuest: {
      name: "AbortQuest",
      requestType: AbortQuestRequest,
      requestStream: false,
      responseType: AbortQuestResponse,
      responseStream: false,
      options: {},
    },
    sendEvent: {
      name: "SendEvent",
      requestType: Event,
      requestStream: false,
      responseType: EventResponse,
      responseStream: false,
      options: {},
    },
    /** Listen to changes in quest states and event processing updates */
    subscribe: {
      name: "Subscribe",
      requestType: User,
      requestStream: false,
      responseType: UserUpdate,
      responseStream: true,
      options: {},
    },
  },
} as const

type Builtin = Date | Function | Uint8Array | string | number | boolean | undefined

export type DeepPartial<T> = T extends Builtin
  ? T
  : T extends Array<infer U>
  ? Array<DeepPartial<U>>
  : T extends ReadonlyArray<infer U>
  ? ReadonlyArray<DeepPartial<U>>
  : T extends {}
  ? { [K in keyof T]?: DeepPartial<T[K]> }
  : Partial<T>

type KeysOfUnion<T> = T extends T ? keyof T : never
export type Exact<P, I extends P> = P extends Builtin
  ? P
  : P & { [K in keyof P]: Exact<P[K], I[K]> } & { [K in Exclude<keyof I, KeysOfUnion<P>>]: never }

function isObject(value: any): boolean {
  return typeof value === "object" && value !== null
}

function isSet(value: any): boolean {
  return value !== null && value !== undefined
}
