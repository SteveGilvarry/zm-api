import { Injectable } from '@nestjs/common';
import { ConfigCreateInput } from '../@generated/prisma-nestjs-graphql/config/config-create.input';
import { ConfigUpdateInput } from '../@generated/prisma-nestjs-graphql/config/config-update.input';
import { ConfigWhereUniqueInput } from '../@generated/prisma-nestjs-graphql/config/config-where-unique.input';
import { PrismaService } from '../../prisma/prisma.service';

@Injectable()
export class ConfigService {
  constructor(private prisma: PrismaService) {}

  create(createConfigInput: ConfigCreateInput) {
    return this.prisma.config.create({
      data: createConfigInput,
    });
  }

  findAll() {
    return this.prisma.config.findMany();
  }

  findOne(configWhereUniqueInput: ConfigWhereUniqueInput) {
    return this.prisma.config.findUnique({
      where: configWhereUniqueInput,
    });
  }

  update(
    configWhereUniqueInput: ConfigWhereUniqueInput,
    updateConfigInput: ConfigUpdateInput
  ) {
    return this.prisma.config.update({
      where: configWhereUniqueInput,
      data: updateConfigInput,
    });
  }

  remove(id: number) {
    return `This action removes a #${id} config`;
  }
}
