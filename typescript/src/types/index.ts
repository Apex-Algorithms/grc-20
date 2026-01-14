// ID types
export type { Id } from "./id.js";
export {
  createId,
  copyId,
  NIL_ID,
  idsEqual,
  compareIds,
} from "./id.js";

// Value types
export type {
  Value,
  PropertyValue,
  Property,
  DecimalMantissa,
} from "./value.js";
export {
  DataType,
  EmbeddingSubType,
  embeddingBytesForDims,
  valueDataType,
  validateValue,
} from "./value.js";

// Operation types
export type {
  Op,
  CreateEntity,
  UpdateEntity,
  DeleteEntity,
  RestoreEntity,
  CreateRelation,
  UpdateRelation,
  DeleteRelation,
  RestoreRelation,
  CreateProperty,
  UnsetLanguage,
  UnsetProperty,
  UnsetRelationField,
} from "./op.js";
export {
  opTypeCode,
  validatePosition,
  OP_TYPE_CREATE_ENTITY,
  OP_TYPE_UPDATE_ENTITY,
  OP_TYPE_DELETE_ENTITY,
  OP_TYPE_RESTORE_ENTITY,
  OP_TYPE_CREATE_RELATION,
  OP_TYPE_UPDATE_RELATION,
  OP_TYPE_DELETE_RELATION,
  OP_TYPE_RESTORE_RELATION,
  OP_TYPE_CREATE_PROPERTY,
} from "./op.js";

// Edit types
export type { Edit, WireDictionaries } from "./edit.js";
export { createWireDictionaries } from "./edit.js";
