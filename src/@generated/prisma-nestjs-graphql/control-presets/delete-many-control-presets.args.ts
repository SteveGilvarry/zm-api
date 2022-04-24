import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ControlPresetsWhereInput } from './control-presets-where.input';

@ArgsType()
export class DeleteManyControlPresetsArgs {

    @Field(() => ControlPresetsWhereInput, {nullable:true})
    where?: ControlPresetsWhereInput;
}
