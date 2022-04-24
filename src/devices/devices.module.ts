import { Module } from '@nestjs/common';
import { DevicesService } from './devices.service';
import { DevicesResolver } from './devices.resolver';
import { PrismaService} from '../../prisma/prisma.service';

@Module({
  providers: [PrismaService, DevicesResolver, DevicesService]
})
export class DevicesModule {}
