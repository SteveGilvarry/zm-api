import { Injectable } from '@nestjs/common';
import { ZonesCreateInput } from '../@generated/prisma-nestjs-graphql/zones/zones-create.input';
import { ZonesUpdateInput } from '../@generated/prisma-nestjs-graphql/zones/zones-update.input';
import { ZonesWhereUniqueInput } from '../@generated/prisma-nestjs-graphql/zones/zones-where-unique.input';
import { PrismaService} from '../../prisma/prisma.service';

@Injectable()
export class ZonesService {
  constructor(private prisma: PrismaService) {}

  create(createZoneInput: ZonesCreateInput) {
    return this.prisma.zones.create({
      data: createZoneInput
    });
  }

  findAll() {
    return this.prisma.zones.findMany();
  }

  findOne(zonesWhereUniqueInput: ZonesWhereUniqueInput) {
    return this.prisma.zones.findUnique({
      where: zonesWhereUniqueInput
    });
  }

  update(
    zonesWhereUniqueInput: ZonesWhereUniqueInput,
    updateZoneInput: ZonesUpdateInput
  ) {
    return this.prisma.zones.update({
      where: zonesWhereUniqueInput,
      data: updateZoneInput
    });
  }

  remove(zonesWhereUniqueInput: ZonesWhereUniqueInput) {
    return this.prisma.zones.delete({
      where: zonesWhereUniqueInput
    });
  }
}
