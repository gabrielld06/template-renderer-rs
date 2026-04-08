const componentConfig = {
  raw: "{{componentName}}",
  camelCase: "{{componentName_camelCase}}",
  PascalCase: "{{componentName_PascalCase}}",
  snake_case: "{{componentName_snake_case}}",
  "kebab-name": "{{componentName_kebab-case}}",
  UPPER_CASE: "{{componentName_UPPER_CASE}}"
};

export function create{{componentName_PascalCase}}Message() {
  return `Hello, {{greetingTarget}}. Loaded ${componentConfig.PascalCase}.`;
}

console.log(create{{componentName_PascalCase}}Message());
