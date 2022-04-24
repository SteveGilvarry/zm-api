import { Injectable } from '@nestjs/common';
import { ControlPresetsCreateInput } from '../@generated/prisma-nestjs-graphql/control-presets/control-presets-create.input';
import { ControlPresetsUpdateInput } from '../@generated/prisma-nestjs-graphql/control-presets/control-presets-update.input';
import { ControlPresetsWhereUniqueInput } from '../@generated/prisma-nestjs-graphql/control-presets/control-presets-where-unique.input';
import { PrismaService } from '../../prisma/prisma.service';

@Injectable()
export class ControlpresetsService {
  constructor(private prisma: PrismaService) {}

  create(createControlpresetInput: ControlPresetsCreateInput) {
    return this.prisma.controlPresets.create({
      data: createControlpresetInput,
    });
  }

  findAll() {
    return this.prisma.controlPresets.findMany();
  }

  findOne(controlpresetWhereUniqueInput: ControlPresetsWhereUniqueInput) {
    return this.prisma.controlPresets.findUnique({
      where: controlpresetWhereUniqueInput,
    });
  }

  update(
    controlpresetWhereUniqueInput: ControlPresetsWhereUniqueInput,
    updateControlpresetInput: ControlPresetsUpdateInput
  ) {
    return this.prisma.controlPresets.update({
      where: controlpresetWhereUniqueInput,
      data: updateControlpresetInput,
    });
  }

  remove(id: number) {
    return `This action removes a #${id} controlpreset`;
  }
}
