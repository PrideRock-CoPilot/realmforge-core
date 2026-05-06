// ─────────────────────────────────────────────
// Intake Wizard — Decision Tree Engine
// Evaluates conditions, walks the tree, and
// maps features to modules/personas/skills.
// All logic is deterministic — no AI dependency.
// ─────────────────────────────────────────────

import type {
  IntakeTree,
  IntakeQuestion,
  ConditionRule,
  AnswerMap,
  IdentifiedModule,
  IdentifiedPersona,
  IdentifiedSkill,
} from './types'

// ── Condition Evaluation ──

function resolveOperand(operand: unknown, answers: AnswerMap): unknown {
  if (typeof operand === 'string' && operand.startsWith('question:')) {
    const qId = operand.slice('question:'.length)
    return answers[qId]
  }
  return operand
}

function evaluateCondition(rule: ConditionRule, answers: AnswerMap): boolean {
  const { operator, operands, conditions } = rule

  switch (operator) {
    case 'equals': {
      if (!operands || operands.length < 2) return false
      const [a, b] = operands
      return resolveOperand(a, answers) === resolveOperand(b, answers)
    }

    case 'gt': {
      if (!operands || operands.length < 2) return false
      const a = Number(resolveOperand(operands[0], answers))
      const b = Number(resolveOperand(operands[1], answers))
      return a > b
    }

    case 'gte': {
      if (!operands || operands.length < 2) return false
      const a = Number(resolveOperand(operands[0], answers))
      const b = Number(resolveOperand(operands[1], answers))
      return a >= b
    }

    case 'lt': {
      if (!operands || operands.length < 2) return false
      const a = Number(resolveOperand(operands[0], answers))
      const b = Number(resolveOperand(operands[1], answers))
      return a < b
    }

    case 'lte': {
      if (!operands || operands.length < 2) return false
      const a = Number(resolveOperand(operands[0], answers))
      const b = Number(resolveOperand(operands[1], answers))
      return a <= b
    }

    case 'and': {
      if (!conditions) return true
      return conditions.every((c) => evaluateCondition(c, answers))
    }

    case 'or': {
      if (!conditions) return false
      return conditions.some((c) => evaluateCondition(c, answers))
    }

    case 'not': {
      if (!conditions || conditions.length < 1) return true
      return !evaluateCondition(conditions[0], answers)
    }

    case 'in': {
      if (!operands || operands.length < 2) return false
      const value = resolveOperand(operands[0], answers)
      const arr = resolveOperand(operands[1], answers)
      return Array.isArray(arr) && arr.includes(value)
    }

    default:
      console.warn(`Unknown condition operator: ${operator}`)
      return false
  }
}

// ── Question Visibility ──

export function isQuestionVisible(question: IntakeQuestion, answers: AnswerMap): boolean {
  if (!question.conditions) return true

  const { if_, else_hide } = question.conditions
  const conditionMet = evaluateCondition(if_, answers)

  if (conditionMet) return true
  if (else_hide) return false
  return true // show by default if else_hide is false
}

// ── Get visible questions in order ──

export function getVisibleQuestions(tree: IntakeTree, answers: AnswerMap): IntakeQuestion[] {
  return tree.questions
    .filter((q) => isQuestionVisible(q, answers))
    .sort((a, b) => a.order - b.order)
}

// ── Get current question index ──

export function getCurrentQuestionIndex(tree: IntakeTree, answers: AnswerMap): number {
  const visible = getVisibleQuestions(tree, answers)
  // Find the first unanswered required question
  const idx = visible.findIndex((q) => {
    const val = answers[q.question_id]
    return val === undefined || val === null || val === ''
  })
  return idx === -1 ? visible.length - 1 : idx
}

// ── Get current question ──

export function getCurrentQuestion(tree: IntakeTree, answers: AnswerMap): IntakeQuestion | null {
  const visible = getVisibleQuestions(tree, answers)
  const idx = getCurrentQuestionIndex(tree, answers)
  return visible[idx] ?? null
}

// ── Feature Mapping ──

export interface MappedFeatures {
  modules: IdentifiedModule[]
  personas: IdentifiedPersona[]
  skills: IdentifiedSkill[]
}

export function mapFeatures(tree: IntakeTree, answers: AnswerMap): MappedFeatures {
  const modules: IdentifiedModule[] = []
  const personas: IdentifiedPersona[] = []
  const skills: IdentifiedSkill[] = []

  // Direct feature mappings
  for (const mapping of tree.feature_mappings) {
    const answer = answers[mapping.question_id]

    // Check if answer matches the mapping value
    let matches = false
    if (typeof mapping.value === 'boolean') {
      matches = answer === mapping.value
    } else if (typeof mapping.value === 'number') {
      matches = Number(answer) === mapping.value
    } else {
      matches = String(answer) === mapping.value
    }

    if (matches) {
      for (const m of mapping.modules) {
        modules.push({ module: m, source: `${mapping.question_id}=${mapping.value}`, isMeta: false })
      }
      for (const p of mapping.personas) {
        personas.push({ persona: p, source: `${mapping.question_id}=${mapping.value}`, isMeta: false })
      }
      for (const s of mapping.skills) {
        skills.push({ skill: s, source: `${mapping.question_id}=${mapping.value}`, isMeta: false })
      }
    }
  }

  // Meta mappings (cross-cutting concerns)
  for (const [key, meta] of Object.entries(tree.meta_mappings)) {
    if (evaluateCondition(meta.condition, answers)) {
      for (const m of meta.modules) {
        modules.push({ module: m, source: `meta:${key}`, isMeta: true })
      }
      for (const p of meta.personas) {
        personas.push({ persona: p, source: `meta:${key}`, isMeta: true })
      }
      for (const s of meta.skills) {
        skills.push({ skill: s, source: `meta:${key}`, isMeta: true })
      }
    }
  }

  // Deduplicate
  return {
    modules: dedupeByIdentifier(modules, 'module'),
    personas: dedupeByIdentifier(personas, 'persona'),
    skills: dedupeByIdentifier(skills, 'skill'),
  }
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function dedupeByIdentifier<T extends Record<string, any>>(
  arr: T[],
  key: keyof T
): T[] {
  const seen = new Set<unknown>()
  return arr.filter((item) => {
    const id = item[key]
    if (seen.has(id)) return false
    seen.add(id)
    return true
  })
}

// ── Progress ──

export function getProgress(tree: IntakeTree, answers: AnswerMap): {
  answered: number
  total: number
  percent: number
} {
  const visible = getVisibleQuestions(tree, answers)
  const answered = visible.filter((q) => {
    const val = answers[q.question_id]
    return val !== undefined && val !== null && val !== ''
  }).length
  const total = visible.length
  return {
    answered,
    total,
    percent: total > 0 ? Math.round((answered / total) * 100) : 0,
  }
}

// ── Generate plan name from answers ──

export function generatePlanName(tree: IntakeTree, answers: AnswerMap): string {
  const sourceIds = tree.plan_stage_handoff.generate_name_from
  const parts: string[] = []

  for (const qId of sourceIds) {
    const answer = answers[qId]
    if (answer !== undefined && answer !== null && answer !== '') {
      const question = tree.questions.find((q) => q.question_id === qId)
      const option = question?.options?.find((o) => o.value === String(answer))
      const label = option?.label ?? String(answer)
      parts.push(label)
    }
  }

  if (parts.length === 0) {
    return `Intake — ${tree.name}`
  }

  return `${parts.join(' — ')} (${tree.name})`
}

// ── Find matching tree ──

export function findTreeForAppType(trees: IntakeTree[], appTypeId: string): IntakeTree | null {
  return trees.find((t) => t.applies_to.includes(appTypeId)) ?? null
}
