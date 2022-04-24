import { Injectable } from '@nestjs/common';
import { ManufacturersCreateInput} from '../@generated/prisma-nestjs-graphql/manufacturers/manufacturers-create.input';
import { ManufacturersUpdateInput } from '../@generated/prisma-nestjs-graphql/manufacturers/manufacturers-update.input';
import { ManufacturersWhereUniqueInput } from '../@generated/prisma-nestjs-graphql/manufacturers/manufacturers-where-unique.input';
import { PrismaService} from '../../prisma/prisma.service';

@Injectable()
export class ManufacturersService {
  constructor(private prisma: PrismaService) {}

  create(createManufacturerInput: ManufacturersCreateInput) {
    return this.prisma.manufacturers.create({
      data: createManufacturerInput,
    });
  }

  findAll() {
    return this.prisma.manufacturers.findMany();
  }

  findOne(manufacturerWhereUniqueInput: ManufacturersWhereUniqueInput) {
    return this.prisma.manufacturers.findMany({
      where: manufacturerWhereUniqueInput,
    });
  }

  update(
    manufacturerWhereUniqueInput: ManufacturersWhereUniqueInput,
    updateManufacturerInput: ManufacturersUpdateInput
  ) {
    return this.prisma.manufacturers.update({
      where: manufacturerWhereUniqueInput,
      data: updateManufacturerInput,
    });
  }

  remove(manufacturerWhereUniqueInput: ManufacturersWhereUniqueInput) {
    return this.prisma.manufacturers.delete({
      where: manufacturerWhereUniqueInput,
    });
  }
}
