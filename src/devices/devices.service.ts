import { Injectable } from '@nestjs/common';
import { DevicesCreateInput}from '../@generated/prisma-nestjs-graphql/devices/devices-create.input';
import { DevicesUpdateInput}from '../@generated/prisma-nestjs-graphql/devices/devices-update.input';
import { DevicesWhereUniqueInput}from '../@generated/prisma-nestjs-graphql/devices/devices-where-unique.input';
import { PrismaService } from '../../prisma/prisma.service';


@Injectable()
export class DevicesService {
  constructor(private prisma: PrismaService) {}


  create(createDeviceInput: DevicesCreateInput) {
    return this.prisma.devices.create({
      data: createDeviceInput,
    });
  }

  findAll() {
    return this.prisma.devices.findMany();
  }

  findOne(deviceWhereUniqueInput: DevicesWhereUniqueInput) {
    return this.prisma.devices.findUnique({
      where: deviceWhereUniqueInput,
    });
  }

  update(
    devicesWhereUniqueInput: DevicesWhereUniqueInput,
    devicesUpdateInput: DevicesUpdateInput,
  ) {
    return this.prisma.devices.update({
      where: devicesWhereUniqueInput,
      data: devicesUpdateInput,
    });
  }

  remove(deviceWhereUniqueInput: DevicesWhereUniqueInput) {
    return this.prisma.devices.delete({
      where: deviceWhereUniqueInput,
    });
  }
}
