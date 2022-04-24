import { Module } from '@nestjs/common';
import { MonitorsService } from './monitors.service';
import { MonitorsResolver } from './monitors.resolver';
import { PrismaService } from '../../prisma/prisma.service';

@Module({
  providers: [PrismaService, MonitorsResolver, MonitorsService]
})
export class MonitorsModule {}
