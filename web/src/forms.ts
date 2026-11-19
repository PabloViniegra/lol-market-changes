import type { TypedSchema } from "vee-validate";
import type { z } from "zod";

// @vee-validate/zod only supports zod 3, and Zod 4 errors map onto vee-validate paths directly.
export function zodSchema<T extends z.ZodType>(schema: T): TypedSchema<z.input<T>, z.output<T>> {
  return {
    __type: "VVTypedSchema",
    async parse(values) {
      const result = await schema.safeParseAsync(values);

      if (result.success) return { value: result.data, errors: [] };

      const errors = new Map<string, string[]>();

      for (const issue of result.error.issues) {
        const path = issue.path.join(".");

        errors.set(path, [...(errors.get(path) ?? []), issue.message]);
      }

      return { errors: [...errors].map(([path, messages]) => ({ path, errors: messages })) };
    },
  };
}
