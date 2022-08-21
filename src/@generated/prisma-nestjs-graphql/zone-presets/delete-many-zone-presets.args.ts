import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ZonePresetsWhereInput } from './zone-presets-where.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteManyZonePresetsArgs {

    @Field(() => ZonePresetsWhereInput, {nullable:true})
    @Type(() => ZonePresetsWhereInput)
    where?: ZonePresetsWhereInput;
}
