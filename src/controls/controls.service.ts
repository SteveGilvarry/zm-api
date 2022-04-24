import { Injectable } from '@nestjs/common';
import { ControlsCreateInput } from '../@generated/prisma-nestjs-graphql/controls/controls-create.input';
import { ControlsUpdateInput } from '../@generated/prisma-nestjs-graphql/controls/controls-update.input';
import { ControlsWhereUniqueInput} from '../@generated/prisma-nestjs-graphql/controls/controls-where-unique.input';
import { PrismaService} from '../../prisma/prisma.service';

@Injectable()
export class ControlsService {
  constructor(private prisma: PrismaService) {}

  create(createControlInput: ControlsCreateInput) {
    return this.prisma.controls.create({
      data: createControlInput,
    });
  }

  findAll() {
    return this.prisma.controls.findMany();
  }

  findOne(controlsWhereUniqueInput: ControlsWhereUniqueInput) {
    return this.prisma.controls.findUnique({
      where: controlsWhereUniqueInput,
    });
  }

  update(
    controlsWhereUniqueInput: ControlsWhereUniqueInput,
    updateControlInput: ControlsUpdateInput
  ) {
    return this.prisma.controls.update({
      where: controlsWhereUniqueInput,
      data: updateControlInput
    });
  }

  remove(controlsWhereUniqueInput: ControlsWhereUniqueInput) {
    return this.prisma.controls.delete({
      where: controlsWhereUniqueInput
    });
  }
}
