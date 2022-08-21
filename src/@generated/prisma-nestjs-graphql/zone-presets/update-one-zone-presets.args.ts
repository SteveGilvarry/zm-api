import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ZonePresetsUpdateInput } from './zone-presets-update.input';
import { Type } from 'class-transformer';
import { ZonePresetsWhereUniqueInput } from './zone-presets-where-unique.input';

@ArgsType()
export class UpdateOneZonePresetsArgs {

    @Field(() => ZonePresetsUpdateInput, {nullable:false})
    @Type(() => ZonePresetsUpdateInput)
    data!: ZonePresetsUpdateInput;

    @Field(() => ZonePresetsWhereUniqueInput, {nullable:false})
    @Type(() => ZonePresetsWhereUniqueInput)
    where!: ZonePresetsWhereUniqueInput;
}
