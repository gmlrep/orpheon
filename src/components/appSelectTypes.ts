export interface AppSelectOption<V extends string | number | boolean = string | number | boolean> {
  value: V;
  label: string;
}
