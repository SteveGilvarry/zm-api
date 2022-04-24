import { Injectable } from '@nestjs/common';
import { MontageLayoutsCreateInput } from '../@generated/prisma-nestjs-graphql/montage-layouts/montage-layouts-create.input';
import { MontageLayoutsUpdateInput} from '../@generated/prisma-nestjs-graphql/montage-layouts/montage-layouts-update.input';
import { MontageLayoutsWhereUniqueInput } from '../@generated/prisma-nestjs-graphql/montage-layouts/montage-layouts-where-unique.input';
import { PrismaService } from '../../prisma/prisma.service';

@Injectable()
export class MontagelayoutsService {
  constructor(private prisma: PrismaService) {}

  create(createMontagelayoutInput: MontageLayoutsCreateInput) {
    return this.prisma.montageLayouts.create({
      data: createMontagelayoutInput
    });
  }

  findAll() {
    return this.prisma.montageLayouts.findMany();
  }

  findOne(montageLayoutsWhereUniqueInput: MontageLayoutsWhereUniqueInput) {
    return this.prisma.montageLayouts.findUnique({
      where: montageLayoutsWhereUniqueInput
    });
  }

  update(
    montageLayoutsWhereUniqueInput: MontageLayoutsWhereUniqueInput,
    updateMontagelayoutInput: MontageLayoutsUpdateInput
  ) {
    return this.prisma.montageLayouts.update({
      where: montageLayoutsWhereUniqueInput,
      data: updateMontagelayoutInput
    });
  }

  remove(montageLayoutsWhereUniqueInput: MontageLayoutsWhereUniqueInput) {
    return this.prisma.montageLayouts.delete({
      where: montageLayoutsWhereUniqueInput
    });
  }
}
