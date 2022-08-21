import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ZonePresetsUpdateManyMutationInput } from './zone-presets-update-many-mutation.input';
import { Type } from 'class-transformer';
import { ZonePresetsWhereInput } from './zone-presets-where.input';

@ArgsType()
export class UpdateManyZonePresetsArgs {

    @Field(() => ZonePresetsUpdateManyMutationInput, {nullable:false})
    @Type(() => ZonePresetsUpdateManyMutationInput)
    data!: ZonePresetsUpdateManyMutationInput;

    @Field(() => ZonePresetsWhereInput, {nullable:true})
    @Type(() => ZonePresetsWhereInput)
    where?: ZonePresetsWhereInput;
}
