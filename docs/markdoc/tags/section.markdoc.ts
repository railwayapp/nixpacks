import { Section } from "../../components";

export const section = {
  render: Section,
  description: "Display the enclosed content in a section",
  children: ["paragraph", "tag", "list"],
  attributes: {
    title: {
      type: String,
      description: "The title displayed",
    },
    subtitle: {
      type: String,
      description: "The subtitle displayed",
    },
  },
};
