import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ControlPresetsUpdateManyMutationInput } from './control-presets-update-many-mutation.input';
import { Type } from 'class-transformer';
import { ControlPresetsWhereInput } from './control-presets-where.input';

@ArgsType()
export class UpdateManyControlPresetsArgs {

    @Field(() => ControlPresetsUpdateManyMutationInput, {nullable:false})
    @Type(() => ControlPresetsUpdateManyMutationInput)
    data!: ControlPresetsUpdateManyMutationInput;

    @Field(() => ControlPresetsWhereInput, {nullable:true})
    @Type(() => ControlPresetsWhereInput)
    where?: ControlPresetsWhereInput;
}
