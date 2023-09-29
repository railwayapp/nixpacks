import { PrismaClient } from "@prisma/client";

const prisma = new PrismaClient({});

await prisma.user.create({
  data: { name: "hello" },
});

const users = await prisma.user.findMany();
console.log(users);
