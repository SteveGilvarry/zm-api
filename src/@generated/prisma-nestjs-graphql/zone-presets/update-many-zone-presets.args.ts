import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ZonePresetsUpdateManyMutationInput } from './zone-presets-update-many-mutation.input';
import { ZonePresetsWhereInput } from './zone-presets-where.input';

@ArgsType()
export class UpdateManyZonePresetsArgs {

    @Field(() => ZonePresetsUpdateManyMutationInput, {nullable:false})
    data!: ZonePresetsUpdateManyMutationInput;

    @Field(() => ZonePresetsWhereInput, {nullable:true})
    where?: ZonePresetsWhereInput;
}
