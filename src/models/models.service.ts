import { Injectable } from '@nestjs/common';
import { ModelsCreateInput} from '../@generated/prisma-nestjs-graphql/models/models-create.input';
import { ModelsUpdateInput} from '../@generated/prisma-nestjs-graphql/models/models-update.input';
import { ModelsWhereUniqueInput} from '../@generated/prisma-nestjs-graphql/models/models-where-unique.input';
import { PrismaService} from '../../prisma/prisma.service';

@Injectable()
export class ModelsService {
  constructor(private prisma: PrismaService) {}

  create(createModelInput: ModelsCreateInput) {
    return this.prisma.models.create({
      data: createModelInput
    });
  }

  findAll() {
    return this.prisma.models.findMany();
  }

  findOne(modelsWhereUniqueInput: ModelsWhereUniqueInput) {
    return this.prisma.models.findUnique({
      where: modelsWhereUniqueInput
    });
  }

  update(
    modelsWhereUniqueInput: ModelsWhereUniqueInput,
    updateModelInput: ModelsUpdateInput
  ) {
    return this.prisma.models.update({
      where: modelsWhereUniqueInput,
      data: updateModelInput
    });
  }

  remove(modelsWhereUniqueInput: ModelsWhereUniqueInput) {
    return this.prisma.models.delete({
      where: modelsWhereUniqueInput
    });
  }
}
