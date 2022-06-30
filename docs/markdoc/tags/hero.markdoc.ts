import { Hero } from "../../components";

export const hero = {
  render: Hero,
  description: "Hero section",
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
