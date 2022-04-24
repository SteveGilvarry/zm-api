import { Injectable } from '@nestjs/common';
import { StatesCreateInput } from '../@generated/prisma-nestjs-graphql/states/states-create.input';
import { StatesUpdateInput } from '../@generated/prisma-nestjs-graphql/states/states-update.input';
import { StatesWhereUniqueInput } from '../@generated/prisma-nestjs-graphql/states/states-where-unique.input';
import { PrismaService } from '../../prisma/prisma.service';

@Injectable()
export class StatesService {
  constructor(private prisma: PrismaService) {}

  create(createStateInput: StatesCreateInput) {
    return this.prisma.states.create({
      data: createStateInput
    });
  }

  findAll() {
    return this.prisma.states.findMany();
  }

  findOne(stateWhereUniqueInput: StatesWhereUniqueInput) {
    return this.prisma.states.findUnique({
      where: stateWhereUniqueInput
    });
  }

  update(
    stateWhereUniqueInput: StatesWhereUniqueInput,
    updateStateInput: StatesUpdateInput
  ) {
    return this.prisma.states.update({
      where: stateWhereUniqueInput,
      data: updateStateInput
    });
  }

  remove(stateWhereUniqueInput: StatesWhereUniqueInput) {
    return this.prisma.states.delete({
      where: stateWhereUniqueInput
    });
  }
}
