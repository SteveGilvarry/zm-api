import { Module } from '@nestjs/common';
import { ControlsService } from './controls.service';
import { ControlsResolver } from './controls.resolver';
import { PrismaService } from '../../prisma/prisma.service';

@Module({
  providers: [PrismaService, ControlsResolver, ControlsService]
})
export class ControlsModule {}
