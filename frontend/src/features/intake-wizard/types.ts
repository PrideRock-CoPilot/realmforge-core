// ─────────────────────────────────────────────
// Intake Wizard — Decision Tree Types
// These match the JSON schema at
// catalog/intake-trees/ trees.
// ─────────────────────────────────────────────

export interface IntakeTree {
  tree_id: string
  name: string
  version: string
  description: string
  applies_to: string[]
  max_depth: number
  questions: IntakeQuestion[]
  feature_mappings: FeatureMapping[]
  meta_mappings: Record<string, MetaMapping>
  plan_stage_handoff: PlanStageHandoff
}

export type QuestionTypeName = 'SingleChoice' | 'Boolean' | 'Number'

export interface QuestionType {
  type: QuestionTypeName
  params?: { min?: number; max?: number }
}

export interface QuestionOption {
  value: string
  label: string
}

export interface ConditionRule {
  operator: string
  operands: unknown[]
  conditions?: ConditionRule[]
}

export interface QuestionConditions {
  if_: ConditionRule
  then_show_ai_banner: boolean
  else_hide: boolean
}

export interface IntakeQuestion {
  question_id: string
  question_type: QuestionType
  text: string
  options?: QuestionOption[]
  order: number
  required: boolean
  is_root?: boolean
  conditions?: QuestionConditions
}

export interface FeatureMapping {
  question_id: string
  value: string | boolean | number
  modules: string[]
  personas: string[]
  skills: string[]
}

export interface MetaMapping {
  condition: ConditionRule
  modules: string[]
  personas: string[]
  skills: string[]
}

export interface PlanStageHandoff {
  map_requirements: boolean
  generate_constraints: boolean
  generate_name_from: string[]
  default_scope_template: string
}

// ── Wizard Runtime Types ──

export type WizardStep = 'select-type' | 'questions' | 'summary'

export interface AnswerMap {
  [questionId: string]: string | boolean | number | null
}

export interface AppTypeDefinition {
  type_id: string
  name: string
  description: string
  icon: React.ReactNode
  complexity: string
}


export interface IdentifiedModule {
  module: string
  source: string // which question/value triggered this
  isMeta: boolean
}

export interface IdentifiedPersona {
  persona: string
  source: string
  isMeta: boolean
}

export interface IdentifiedSkill {
  skill: string
  source: string
  isMeta: boolean
}
